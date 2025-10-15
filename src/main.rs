use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bizppurio::{Body, SMSClient};
use chrono::Utc;
use mqtt_client::{client::MqttClient,info::MqttClientInfoBuilder};
use rand::{random};
use tokio::time::{sleep,Duration};
use dolphin::{convert_str, database_conn, datetime_now_format, decompose_id, env_vars, get_all_equipment_status, get_ampere_data, get_equipments, logger_log, opt_or_empty, print_version_info, send_sms};
use dolphin::models::EquipmentStatusState;
use dolphin::models::NewDisplayDeviceInfo;

#[derive(Debug, Clone)]
struct EquipmentLightData {
    pub red: HashMap<String, f32>,
    pub green: HashMap<String, f32>
}

type Collection = Arc<Mutex<HashMap<String, EquipmentLightData>>>;

fn get_phone_numbers() -> Vec<String> {
    match std::env::var("ALERT_NUMBERS") {
        Ok(phone_numbers) => parse_comma_seperated(&phone_numbers),
        Err(_) => Vec::new(),
    }
}

fn get_exclude_devices() -> Vec<String> {
    match std::env::var("EXCLUDE_DEVICES") {
        Ok(devices) => parse_comma_seperated(&devices),
        Err(_) => Vec::new(),
    }
}

fn parse_comma_seperated(phone_numbers: &str) -> Vec<String> {
    phone_numbers
        .split(',')
        .map(|phone_number| phone_number.trim().to_string())
        .collect()
}

async fn send_sms_all(message: &str) {
    let account_id = std::env::var("BIZ_SMS_ACCOUNT_ID").unwrap_or_default();
    let secret_key = std::env::var("BIZ_SMS_SECRET_KEY").unwrap_or_default();
    let from_number = std::env::var("BIZ_SMS_FROM").unwrap_or_default();

    let mut client = SMSClient::new(account_id, secret_key, from_number);
    for phone_number in get_phone_numbers() {
        if let Err(err) = client.send_to(&phone_number, Body::SMS(String::from(message))).await {
            logger_log!("SMSService", format!("Failed to send SMS to {} \n{}", phone_number, err));
        }
    }
}

fn set_default_i32(is_abnormal: &mut bool) -> i32 {
    *is_abnormal = true;
    0
}

fn set_default_f32(is_abnormal: &mut bool) -> f32 {
    *is_abnormal = true;
    0f32
}

fn get_tolerance(duty_rate: i32) -> f32 {
    if duty_rate == 100 {
        0.2
    } else if duty_rate > 50 {
        0.3
    } else if duty_rate > 10 {
        0.4
    } else {
        0.5
    }
}

async fn update_collection_map(collection: Collection) {
    let mut collection = collection.lock().unwrap();
    if let Some(mut conn) = database_conn() {
        for (equipment_id, equipment_type, _, _, units, _) in get_equipments(&mut conn) {
            let formatted_id = format!("{equipment_type}{equipment_id}");
            let mut light_data = EquipmentLightData {
                red: HashMap::new(),
                green: HashMap::new()
            };
            for is_red in [true, false].iter() {
                let mut duty_map: HashMap<String, f32> = HashMap::new();
                let mut count_map: HashMap<String, i32> = HashMap::new();
                for (ampere, duty_rate) in get_ampere_data(&mut conn, equipment_id, equipment_type.as_str(), *is_red) {
                    if ampere != 0f32 && duty_rate != 0f32 {
                        *duty_map.entry(duty_rate.to_string()).or_insert(0f32) += ampere;
                        *count_map.entry(duty_rate.to_string()).or_insert(0) += 1;
                    }
                }
                // calculate average value
                for (key, total) in duty_map.iter() {
                    let count = *count_map.get(key).unwrap_or(&0);
                    if count != 0 {
                        let average = if units > 0 {
                            (total / count as f32) / units as f32
                        } else {
                            0f32
                        };
                        if *is_red {
                            *light_data.red.entry((*key).clone()).or_insert(0f32) = average;
                        } else {
                            *light_data.green.entry((*key).clone()).or_insert(0f32) = average;
                        }
                    }
                }
                // clear
                duty_map.clear();
                count_map.clear();
            }

            // update collection
            if let Some(value) = collection.get_mut(formatted_id.as_str()) {
                *value = light_data;
            } else {
                collection.insert(formatted_id.clone(), light_data);
            }
        }
    }
}

async fn update_equipment_state() {
    let mut conn = database_conn().unwrap();
    for (id, equipment_type, device_state, interval_time, _, place_name) in get_equipments(&mut  conn).iter() {
        let device_state = device_state.to_uppercase();
        let place_name = place_name.clone();
        let interval_time = *interval_time as f64;
        let logs = get_all_equipment_status(&mut conn, (*equipment_type).as_str(), *id);
        if !logs.is_empty() {
            let (_, _, _, _, receive_date) = logs.first().unwrap();
            let korea_timezone = chrono::offset::FixedOffset::east_opt(9 * 3600).unwrap();
            let receive_date = (*receive_date).and_local_timezone(korea_timezone).unwrap();
            let receive_date: chrono::DateTime<Utc> = chrono::DateTime::from_naive_utc_and_offset(receive_date.naive_utc(), Utc);
            let current_date = Utc::now();
            let differ_time = current_date.signed_duration_since(receive_date).num_seconds();

            if (differ_time as f64) > interval_time * 1.5 {
                if device_state.ne("FAULT") {
                    dolphin::update_equipment_state(&mut conn, *id, equipment_type, "FAULT");
                    let message = format!("'{}' 장소에 설치된 장비({}-{}) \n셀룰러(LTE) 오류가 발생했습니다.", place_name, equipment_type, id);
                    send_sms_all(message.as_ref()).await;
                }
            } else if device_state.eq("FAULT") {
                dolphin::update_equipment_state(&mut conn, *id, equipment_type, "NORMAL");
                let message = format!("'{}' 장소에 설치된 장비({}-{}) \n셀룰러(LTE)가 재개되었습니다.", place_name, equipment_type, id);
                send_sms_all(message.as_ref()).await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    print_version_info!("dolphin", "0.2.2");
    let client_id = (random::<f64>() * 1e16) as u64;
    let mut mqtt_host= String::new();
    let mut username = String::new();
    let mut password = String::new();
    env_vars!(
        ("MQTT_HOST", &mut mqtt_host, true),
        ("MQTT_USERNAME", &mut username, false),
        ("MQTT_PASSWORD", &mut password, false),
    );

    logger_log!("DolphinFactory", "Starting Dolphin application...");

    let info = MqttClientInfoBuilder::new()
        .server_uri(format!("mqtt://{}", mqtt_host).as_str())
        .client_id(format!("dolphin-{:x}", client_id).as_str())
        .username(username.as_str())
        .password(password.as_str())
        .finalize();
    let cli = MqttClient::new(info).unwrap();
    let collection: Collection = Arc::new(Mutex::new(HashMap::new()));
    let main_cli = Arc::new(tokio::sync::Mutex::new(cli));
    let mut main_client = main_cli.lock().await.clone();

    let cli = Arc::clone(&main_cli);
    main_client.on_connect(move || {
        let cli_clone = Arc::clone(&cli);
        tokio::spawn(async move {
            use chrono::{FixedOffset, Utc};
            sleep(Duration::from_secs_f64(0.125)).await;
            send_sms_all("MQTT Broker 정상적으로 연결되었습니다.").await;
            let _ = database_conn();
            loop {
                let client = cli_clone.lock().await;
                if client.is_connected() {
                    if let Some(time_zone) = FixedOffset::east_opt(9 * 3600) {
                        let date_time = Utc::now().with_timezone(&time_zone);
                        let formatted = date_time.format("%m%d%H%M").to_string();

                        client.publish("timestamp", &formatted);
                        drop(client);
                        sleep(Duration::from_secs(5 * 60)).await;
                    }
                } else {
                    drop(client);
                    sleep(Duration::from_secs_f64(0.125)).await;
                }
            }
        });

        let cli_clone = Arc::clone(&cli);
        tokio::spawn(async move {
            sleep(Duration::from_secs_f64(0.125)).await;
            loop {
                let client = cli_clone.lock().await;
                if client.is_connected() {
                    client.publish("beacon", "ping");
                    drop(client);
                    sleep(Duration::from_secs(5 * 60)).await;
                } else {
                    drop(client);
                    sleep(Duration::from_secs_f64(0.125)).await;
                }
            }
        });
    });

    let collection_clone = collection.clone();
    main_client.subscribe("+/status/controller",move |topic, payload| {
        let fields = convert_str(payload).split('\n').map(|s| s.to_string()).collect::<Vec<String>>();
        let message = fields.join(" | ");
        logger_log!("MqttService", format!("Topic('{topic}'): {message}"));

        if fields.len() != 19 {
            return;
        }

        let id = topic.split('/').collect::<Vec<&str>>().first().copied().unwrap_or_default();
        let (equip_type, equip_id) = decompose_id(id);
        if equip_type.trim() != "" && equip_id > 0 {
            use dolphin::{get_equipment};
            use dolphin::{create_equipment_status, update_equipment_state};

            let mut is_abnormal = false;

             let vol_red = opt_or_empty!(fields.first()).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let vol_green = opt_or_empty!(fields.get(1)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));

            let ampere_red = opt_or_empty!(fields.get(2)).parse().unwrap_or_else(|_| set_default_f32(&mut is_abnormal));
            let ampere_green = opt_or_empty!(fields.get(3)).parse().unwrap_or_else(|_| set_default_f32(&mut is_abnormal));
            let ampere_off = opt_or_empty!(fields.get(4)).parse().unwrap_or_else(|_| set_default_f32(&mut is_abnormal));
            
            let duty_red = opt_or_empty!(fields.get(5)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let duty_green = opt_or_empty!(fields.get(6)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let output_status = opt_or_empty!(fields.get(7)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let temperature = opt_or_empty!(fields.get(8)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let power_limit = opt_or_empty!(fields.get(9)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let direction = opt_or_empty!(fields.get(10)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let operation = opt_or_empty!(fields.get(11)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let rs485 = opt_or_empty!(fields.get(12)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let publish_count = opt_or_empty!(fields.get(13)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let reset_count = opt_or_empty!(fields.get(14)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let unit_comm_status = opt_or_empty!(fields.get(15)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let status_for_unit = opt_or_empty!(fields.get(16)).replace(' ', "");
            let controller_ver = opt_or_empty!(fields.get(17)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
            let controller_time = opt_or_empty!(fields.get(18)).parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));

            let raw_temperature = {
                let raw = temperature as i64;

                // Handle signed 32-bit two's complement
                if raw & (1 << 31) != 0 {
                    (raw - (1i64 << 32)) as i32
                } else {
                    raw as i32
                }
            };

            // check a negative bit
            // let temperature = {
            //     let temperature = temperature as i64 * 10;
            //     if temperature >> 31 == 1 {
            //         // take a Two's complement and convert it to a negative number
            //         ((temperature - 2i64.pow(32)) as f32 / 10f32).round() as i32
            //     } else {
            //         (temperature / 10) as i32
            //     }
            // };

            // let temperature = {
            //     let raw = temperature as i64;

            //     // Handle signed 32-bit two's complement if necessary
            //     let signed_temp = if raw >> 31 == 1 {
            //         raw - (1i64 << 32)
            //     } else {
            //         raw
            //     };

            //     ((signed_temp - 400) as f32 / 10.0).round() as i32
            // };

            logger_log!("MqttService", format!("temperature is {temperature}."));

            if let Some(mut conn) = database_conn() {
                let equipment = get_equipment(&mut conn, equip_id, equip_type);
                if let Some((equipment_id, equipment_type, device_state, units, _, _, _, place_name)) = equipment.first() {
                    let mut metric_state = EquipmentStatusState::Normal;
                    let equipment_id = *equipment_id;
                    let equipment_type = equipment_type.to_uppercase();
                    let device_state = device_state.to_uppercase();
                    let units = *units;
                    let place_name = place_name.clone();

                    if device_state.eq("FAULT") {
                        let message = format!("'{place_name}' 장소에 설치된 장비({}-{}) \n셀룰러(LTE)가 재개되었습니다.", equipment_type, equipment_id);
                        tokio::spawn(async move {
                            send_sms_all(message.as_str()).await;
                        });
                    }

                    // classified state (normal / fault / abnormal)
                    if !is_abnormal {
                        let red_ampere_per_unit = if units > 0 {
                            ampere_red / units as f32
                        } else {
                            0f32
                        };
                        let green_ampere_per_unit = if units > 0 {
                            ampere_green / units as f32
                        } else {
                            0f32
                        };
                        let mut metric_normal = if (red_ampere_per_unit == 0f32 && duty_red > 0) || (green_ampere_per_unit == 0f32 && duty_green > 0) {
                            false
                        } else {
                            !((red_ampere_per_unit > 0f32 && duty_red == 0) || (green_ampere_per_unit > 0f32 && duty_green == 0))
                        };
                        let tolerance_red = get_tolerance(duty_red);
                        let tolerance_green = get_tolerance(duty_green);

                        let mut should_send_sms = false;
                        let mut sms_message = "".to_string();

                        let collection = collection_clone.lock().unwrap();
                        if let Some(light_data) = collection.get(id) {
                            if !light_data.red.is_empty() && !light_data.green.is_empty() {
                                let red_avg = *light_data.red.get(&duty_red.to_string()).unwrap_or(&0f32);
                                let green_avg = *light_data.green.get(&duty_green.to_string()).unwrap_or(&0f32);

                                let red_lower_bound = red_avg - red_avg * tolerance_red;
                                let red_upper_bound = red_avg + red_avg * tolerance_red;
                                let green_lower_bound = green_avg - green_avg * tolerance_green;
                                let green_upper_bound = green_avg + green_avg * tolerance_green;
                                    //&& (duty_green > 0 || duty_red > 0)
                                let comm_status = (rs485 == 0 || rs485 == 1 );
                                if comm_status && ((red_lower_bound > red_avg && red_avg > red_upper_bound) || ampere_red == 0f32) {
                                    metric_normal = false;
                                    should_send_sms = true;
                                    sms_message = format!("'{place_name}' 장소에 설치된 장비({}-{}) \n적색등 비정상 전류 \n\n전류: {}mA", equipment_type, equipment_id, ampere_red);
                                    logger_log!("MqttService", format!("[{id}]: 적색등 비정상 전류"));
                                } else if !comm_status {
                                    should_send_sms = true;
                                    sms_message = format!("'{place_name}' 장소에 설치된 장비({}-{}) \n제어부와 RS485 통신 오류가 발생했습니다.", equipment_type, equipment_id);
                                    logger_log!("MqttService", format!("[{id}]: 제어부와 RS485 통신 오류"));
                                } else {
                                    logger_log!("MqttService", format!("[{id}]: 적색등 정상 전류"));
                                }

                                if comm_status && ((green_lower_bound > green_avg && green_avg > green_upper_bound) || ampere_green == 0f32) {
                                    metric_normal = false;
                                    should_send_sms = true;
                                    sms_message = format!("'{place_name}' 장소에 설치된 장비({}-{}) \n녹색등 비정상 전류 \n\n전류: {}mA", equipment_type, equipment_id, ampere_green);
                                    logger_log!("MqttService", format!("[{id}]: 녹색등 비정상 전류"));
                                } else if !comm_status {
                                    should_send_sms = true;
                                    sms_message = format!("'{place_name}' 장소에 설치된 장비({}-{}) \n제어부와 RS485 통신 오류가 발생했습니다.", equipment_type, equipment_id);
                                    logger_log!("MqttService", format!("[{id}]: 제어부와 RS485 통신 오류"));
                                } else {
                                    logger_log!("MqttService", format!("[{id}]: 녹색등 정상 전류"));
                                }
                            }
                        }

                        // exclude device from exclusion list
                        let is_excluded = get_exclude_devices().contains(&format!("{}{}", equipment_type, equipment_id));

                        // send an SMS
                        if should_send_sms && !is_excluded {
                            tokio::spawn(async move {
                                send_sms_all(sms_message.as_str()).await;
                            });
                        }

                        // normal state after all valid metrics pass
                        if metric_normal {
                            metric_state = EquipmentStatusState::Normal;
                            update_equipment_state(&mut conn, equip_id, equip_type, "NORMAL");
                        } else {
                            metric_state = EquipmentStatusState::Abnormal;
                            update_equipment_state(&mut conn, equip_id, equip_type, "ETC");
                        }
                    } else {
                        update_equipment_state(&mut conn, equip_id, equip_type, "ETC");
                        let message = format!("장비({}-{}) 데이터 형식이 맞지가 않습니다.", equipment_type, equipment_id);
                        tokio::spawn(async move {
                            send_sms_all(message.as_ref()).await;
                        });
                    }

                    let payload = format!(
                        "{vol_red}\n{vol_green}\n{ampere_red}\n{ampere_green}\n{ampere_off}\n{duty_red}\n{duty_green}\n{output_status}\n{temperature}\n{power_limit}\n{direction}\n{operation}\n{rs485}\n{publish_count}\n{reset_count}\n{unit_comm_status}\n{status_for_unit}\n{:0>2}\n{:0>8}",
                        controller_ver,
                        controller_time
                    );
                    create_equipment_status(&mut conn, equip_id, equip_type, payload.as_str(), metric_state, is_abnormal);
                }
            }
        }
    });



use std::sync::Arc;
use dolphin::{create_display_device_info, create_display_device_info_firebase};
use dolphin::firedb::DbService;
use dolphin::models::Firedisplayinfo;
use uuid::Uuid;

main_client.subscribe("+/status/dispDevice", {
    let db_service = Arc::new(DbService::new().await);

    move |topic, payload| {
        let db_service = db_service.clone();
        let topic = topic.to_string();
        let payload = payload.to_vec();

        tokio::spawn(async move {
            let fields: Vec<String> = convert_str(&payload)
                .split('\n')
                .flat_map(|s| s.split('|').map(str::trim).filter(|s| !s.is_empty()).map(|s| s.to_string()))
                .collect();

            logger_log!("MqttService", format!("Topic('{topic}'): {}", fields.join(" | ")));

            let id = topic.split('/').next().unwrap_or_default();
            let (equip_type, equip_id) = decompose_id(id);

            if equip_type.trim().is_empty() || equip_id <= 0 {
                logger_log!("MqttService", format!("Invalid equipment ID from topic: {topic}"));
                return;
            }

            let mut chunk_index = 0;
            while chunk_index + 1 < fields.len() {
                let index_no = fields[chunk_index]
                    .parse::<i32>()
                    .unwrap_or_else(|_| {
                        logger_log!("MqttService", "Invalid index number");
                        -1
                    });

                let mut all_data = Vec::new();
                let mut is_abnormal = false;

                for i in 0..4 {
                    let start = chunk_index + 1 + i * 7;
                    if start + 6 >= fields.len() {
                        break;
                    }

                    let led_g = fields[start].parse::<i32>().map(|v| v / 10).unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
                    let led_r = fields[start + 1].parse::<i32>().map(|v| v / 10).unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
                    let cur_g = fields[start + 2].parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
                    let cur_r = fields[start + 3].parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
                    let cur_off_g = fields[start + 4].parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
                    let cur_off_r = fields[start + 5].parse().unwrap_or_else(|_| set_default_i32(&mut is_abnormal));
                    let temp = fields[start + 6].parse::<i64>().map(|raw| {
                        let signed = if raw >> 31 == 1 { raw - (1i64 << 32) } else { raw };
                        ((signed - 400) as f32 / 10.0).round() as i32
                    }).unwrap_or_else(|_| set_default_i32(&mut is_abnormal));

                    all_data.push((led_g, led_r, cur_g, cur_r, cur_off_g, cur_off_r, temp));
                }

                logger_log!("MqttService", format!("Index {} => {} data sets", index_no, all_data.len()));

                let mut conn = database_conn();

                for (i, (led_g, led_r, cur_g, cur_r, cur_off_g, cur_off_r, temp)) in all_data.into_iter().enumerate() {
                    let dataset_index = 4 * (index_no - 1) + i as i32;

                    if let Some(ref mut c) = conn {
                        create_display_device_info(
                            c,
                            &equip_type.to_uppercase(),
                            equip_id,
                            dataset_index,
                            led_g,
                            led_r,
                            cur_g,
                            cur_r,
                            cur_off_g,
                            cur_off_r,
                            temp,
                        );
                    }

                    create_display_device_info_firebase(
                        &db_service,
                        equip_type.to_uppercase(),
                        equip_id,
                        dataset_index,
                        led_g,
                        led_r,
                        cur_g,
                        cur_r,
                        cur_off_g,
                        cur_off_r,
                        temp,
                    ).await;

                    logger_log!(
                        "MqttService",
                        format!(
                            "Saved [{}] G:{} R:{} | Cur G:{} R:{} | Off G:{} R:{} | Temp:{}",
                            dataset_index, led_g, led_r, cur_g, cur_r, cur_off_g, cur_off_r, temp
                        )
                    );
                }

                chunk_index += 29;
            }
        });
    }
});



    

    main_client.on_message(|topic, message| {
        let message = convert_str(message);
        logger_log!("MqttService", format!("Topic('{topic}'): {message}"));
    });

    main_client.on_disconnect(move || {
        logger_log!("MqttService", "Disconnected to Broker!");
        tokio::spawn(async move {
            send_sms_all("MQTT Broker 연결이 끊어졌습니다.").await;
        });
    });

    // sub tasks
    let collection = collection.clone();
    tokio::spawn(async move {
        loop {
            let collection = collection.clone();
            update_collection_map(collection).await;

            sleep(Duration::from_secs(60 * 60)).await;
        }
    });

    // sub tasks
    tokio::spawn(async move {
        loop {
            update_equipment_state().await;

            sleep(Duration::from_secs(5 * 60)).await;
        }
    });

    // main loop
    logger_log!("DolphinApplication", "Dolphin application successfully started");
    loop {
        if !main_client.is_connected() {
            if let Err(err) = main_client.connect().await {
                eprintln!("{}", err);
                sleep(Duration::from_secs(10)).await;
            }
        } else if let Err(err) = main_client.run().await {
            eprintln!("{}", err);
            sleep(Duration::from_secs(10)).await;
        }
    }
}
