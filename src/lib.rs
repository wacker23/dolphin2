pub mod models;
pub mod schema;
pub mod firedb;
use crate::firedb::DbService; 


#[macro_export]
macro_rules! logger_log {
    ($context: expr, $message: expr) => {{
        let context = $context;
        let message = $message;
        let info = format!(
            "\x1B[32m[Dolphin] - \x1B[0m{}\x1B[32m LOG \x1B[33m[{context}]\x1B[32m {message}\x1B[0m",
            datetime_now_format!(),
        );
        println!("{}", info);
    }}
}

#[macro_export]
macro_rules! print_version_info {
    ($name: expr, $version: expr) => {{
        let args: Vec<String> = std::env::args().collect();
        if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
            let name = $name;
            let version = $version;
            let info = format!(
                "\x1B[1m{name}\x1B[0m version: \x1B[1m{name}\x1B[0m/{version} ({})",
                std::env::consts::OS,
            );
            println!("{}", info);
            std::process::exit(0);
        }
    }}
}

#[macro_export]
macro_rules! datetime_now_format {
    () => {
        {
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        }
    }
}

#[macro_export]
macro_rules! env_vars {
    ($(($key:expr, $value:expr, $must_be:expr)),+ $(,)?) => {{
        let mut missing_envs: Vec<&str> = vec![];
        $(
            let env_val = std::env::var($key).unwrap_or_default();
            if !env_val.is_empty() {
                *$value = env_val;
            } else if $must_be {
                missing_envs.push($key);
            }
        )+
        if !missing_envs.is_empty() {
            println!("\x1B[1m\x1B[4mUSAGE:\x1B[0m Must be set \x1B[1m{}\x1B[0m", missing_envs.join(", "));
            std::process::exit(1);
        }
    }}
}

#[macro_export]
macro_rules! opt_or_empty {
    ($opt_value:expr) => {
        if let Some(value) = $opt_value {
            (*value).clone()
        } else {
            "".to_string()
        }
    };
}

#[cfg(test)]
mod tests;

use std::time::{SystemTime, UNIX_EPOCH};
use base64::Engine;
use chrono::NaiveDateTime;
use diesel::{prelude::*, result};
use serde::Deserialize;
use crate::models::{EquipmentStatusState, NewEquipmentStatus};

type HmacSha256 = hmac::Hmac<sha2::Sha256>;

type Equipment = (i32, String, String, i32, i32, i32, i32, String);

pub fn make_timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(unix_time) => {
            unix_time.as_millis().to_string()
        }
        Err(_) => {
            eprintln!("SystemTime before UNIX EPOCH!");
            "".to_string()
        }
    }
}

pub fn make_signature(method: &str, uri: &str, timestamp: &str, access_key: &str, secret_key: &str) -> String {
    use base64::prelude::BASE64_STANDARD;
    use hmac::{Mac};

    let message = format!("{method} {uri}\n{timestamp}\n{access_key}");
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    BASE64_STANDARD.encode(result.into_bytes())
}

#[derive(Debug, Deserialize)]
struct SmsResponse {
    #[allow(dead_code)]
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[allow(dead_code)]
    #[serde(rename = "requestTime")]
    pub request_time: String,
    #[allow(dead_code)]
    #[serde(rename = "statusCode")]
    pub status_code: String,
    #[allow(dead_code)]
    #[serde(rename = "statusName")]
    pub status_name: String
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SmsMessages {
    #[allow(dead_code)]
    pub status_code: String,
    #[allow(dead_code)]
    pub status_name: String,
    #[allow(dead_code)]
    pub messages: Vec<SmsMessage>,
    #[allow(dead_code)]
    pub page_index: i32,
    #[allow(dead_code)]
    pub page_size: i32,
    #[allow(dead_code)]
    pub item_count: i32,
    #[allow(dead_code)]
    pub has_more: bool
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsMessage {
    #[allow(dead_code)]
    pub request_id: String,
    #[allow(dead_code)]
    pub message_id: String,
    #[allow(dead_code)]
    pub request_time: String,
    #[allow(dead_code)]
    pub content_type: String,
    #[allow(dead_code)]
    pub country_code: String,
    #[allow(dead_code)]
    pub from: String,
    #[allow(dead_code)]
    pub to: String,
    #[allow(dead_code)]
    pub status: String,
    #[allow(dead_code)]
    pub status_code: Option<String>,
    #[allow(dead_code)]
    pub status_name: Option<String>,
    #[allow(dead_code)]
    pub status_message: Option<String>,
    pub complete_time: Option<String>,
    pub telco_code: Option<String>,
}

impl SmsMessage {
    pub fn get_complete_time(&self) -> String {
        match self.complete_time.as_deref() {
            Some(complete_time) => complete_time.to_string(),
            None => "".to_string()
        }
    }

    pub fn get_telco_code(&self) -> String {
        match self.telco_code.as_deref() {
            Some("KTF") => "KT".to_string(),
            Some("LGT") => "U+".to_string(),
            Some(other) => other.to_string(),
            None => "".to_string()
        }
    }

    pub fn to_number(&self) -> String {
        let formatted = self.to
            .chars()
            .skip(1)
            .enumerate()
            .fold(String::new(), |mut acc, (i, c)| {
                if i == 2 || i == 6 {
                    acc.push('-');
                }
                acc.push(c);
                acc
            });
        format!("{} +{} {}", self.get_telco_code(), self.country_code, formatted)
    }
}

fn create_request(method: &str, uri: &str) -> reqwest::RequestBuilder {
    let endpoint_url = "https://sens.apigw.ntruss.com";
    let access_key = std::env::var("NCP_ACCESS_KEY").unwrap_or_default();
    let secret_key = std::env::var("NCP_SECRET_KEY").unwrap_or_default();
    let timestamp = make_timestamp();
    let signature = make_signature(
        method,
        uri,
        &timestamp,
        &access_key,
        &secret_key
    );

    let method = reqwest::Method::from_bytes(method.as_bytes()).unwrap();
    let client = reqwest::Client::new();
    client.request(method, format!("{endpoint_url}{uri}"))
        .header("Content-Type", "application/json")
        .header("x-ncp-apigw-timestamp", timestamp)
        .header("x-ncp-iam-access-key", access_key)
        .header("x-ncp-apigw-signature-v2", signature)
}

pub async fn send_sms<'a>(phone_number: &'a str, message: &'a str) -> Result<Option<SmsMessage>, reqwest::Error> {
    let service_id = std::env::var("NCP_SMS_ID").unwrap_or_default();
    let uri = format!("/sms/v2/services/{service_id}/messages");

    // request a http
    let client = create_request("POST", &uri);
    let resp = client
        .json(&serde_json::json!({
            "type": "SMS",
            "countryCode": "82",
            "from": "0415889816",
            "content": message,
            "messages": [
                { "to": phone_number }
            ],
        }))
        .send().await?;
    let text = resp.text().await?;
    if let Ok(json) = serde_json::from_str::<SmsResponse>(&text) {
        if json.status_code == "202" && json.status_name == "success" {
            if let Ok(Some(mut message)) = get_sms_message(&json.request_id).await {
                while message.status != "COMPLETED" {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    if let Ok(Some(new_message)) = get_sms_message(&json.request_id).await {
                        message = new_message;
                    }
                }
                let complete_time = message.get_complete_time();
                logger_log!("SmsService", format!("[{}] SMS({:}) sent to {} successfully", complete_time, message.message_id, message.to_number()));
                return Ok(Some(message))
            }
        }
    } else if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
        if let Some(error) = json.get("error") {
            let error_code = error.get("errorCode").unwrap();
            let details = error.get("details").unwrap();
            if let Some(message) = error.get("message") {
                logger_log!("SmsService", format!("{error_code} - {message}({details})"));
            }
        }
    }
    Ok(None)
}

async fn get_sms_message(request_id: &str) -> Result<Option<SmsMessage>, reqwest::Error> {
    let service_id = std::env::var("NCP_SMS_ID").unwrap_or_default();
    let uri = format!("/sms/v2/services/{service_id}/messages?requestId={}", request_id);

    // request a http
    let client = create_request("GET", &uri);
    let resp = client.send().await?;
    let json = resp.json::<SmsMessages>().await?;
    if json.status_code == "202" && json.status_name == "success" {
        Ok(json.messages.first().cloned())
    } else {
        Ok(None)
    }
}

pub fn decompose_id(id: &str) -> (&str, i32) {
    let equip_types = ["AGL", "DGL", "VGL", "BGL", "LGL"];
    let equip_type = id.chars()
        .take_while(|chr| !chr.is_ascii_digit())
        .collect::<String>();
    let digit = id[equip_type.len()..].parse::<i32>().unwrap_or(0);

    if equip_type.len() != 3 || !equip_types.contains(&equip_type.as_str()) {
        ("", 0)
    } else {
        (&id[..equip_type.len()], digit)
    }
}

pub fn convert_str(msg: &[u8]) -> &str {
    std::str::from_utf8(msg).unwrap_or_default()
}

pub fn is_match_topic(topic: &str, subscription: &str) -> bool {
    let topic_parts = topic.split('/').collect::<Vec<&str>>();
    let sub_parts = subscription.split('/').collect::<Vec<&str>>();
    let is_wildcard = sub_parts.last().unwrap() == &"#";

    if !is_wildcard && sub_parts.len() != topic_parts.len() {
        return false;
    }

    sub_parts.iter().zip(topic_parts.iter()).all(|(sub, topic)| {
        match *sub {
            "#" => true,
            "+" => true,
            _ => sub == topic,
        }
    })
}

pub fn database_conn() -> Option<MysqlConnection> {
    let mut hostname = String::new();
    let mut user_name = String::new();
    let mut password = String::new();
    let mut database = String::new();
    env_vars!(
        ("MARIADB_HOST", &mut hostname, true),
        ("MARIADB_USER", &mut user_name, true),
        ("MARIADB_PASSWORD", &mut password, true),
        ("MARIADB_DATABASE", &mut database, true),
    );

    let database_url = format!("mysql://{user_name}:{password}@{hostname}/{database}");
    let database_url = database_url.as_str();

    match MysqlConnection::establish(database_url) {
        Ok(conn) => {
            logger_log!("database", "successfully connect to the database");
            Some(conn)
        },
        Err(_) => {
            println!("Error connecting to mysql://{user_name}@{hostname}/{database}");
            None
        }
    }
}

pub fn get_equipments(conn: &mut MysqlConnection) -> Vec<(i32, String, String, i32, i32, String)> {
    use crate::schema::EquipmentLocation;
    use crate::schema::Equipment::dsl::*;

    let result = Equipment
        .filter(is_active.eq(true))
        .inner_join(EquipmentLocation::table.on(location_id.eq(EquipmentLocation::id)))
        .select((id, equipment_type, device_state, interval, units, EquipmentLocation::name))
        .order_by(id.asc())
        .load::<(i32, String, String, i32, i32, String)>(conn);
    result.unwrap_or_else(|_| {
        println!("Error loading equipment");
        Vec::default()
    })
}

pub fn get_equipment(conn: &mut MysqlConnection, equip_id: i32, equip_type: &str) -> Vec<Equipment> {
    use crate::schema::EquipmentLocation;
    use crate::schema::Equipment::dsl::*;

    let result = Equipment
        .filter(id.eq(equip_id).and(equipment_type.eq(equip_type)))
        .filter(is_active.eq(true))
        .inner_join(EquipmentLocation::table.on(location_id.eq(EquipmentLocation::id)))
        .select((id, equipment_type, device_state, units, error_cnt, red_correction_cnt, green_correction_cnt, EquipmentLocation::name))
        .load::<(i32, String, String, i32, i32, i32, i32, String)>(conn);
    result.unwrap_or_else(|_| {
        println!("Error loading equipment");
        Vec::default()
    })
}

pub fn update_equipment_state(conn: &mut MysqlConnection, equip_id: i32, equip_type: &str, new_state: &str) -> bool {
    use crate::schema::Equipment::dsl::*;

    let target = Equipment
        .filter(id.eq(equip_id).and(equipment_type.eq(equip_type)));

    let result = diesel::update(target)
        .set(device_state.eq(new_state))
        .execute(conn);
    match result {
        Ok(affected) => {
            logger_log!("SQLService", format!("Update a state('{new_state}') of {}-{:03} to database.", equip_type, equip_id));
            affected == 1
        },
        Err(_) => false,
    }
}

pub fn update_error_count(conn: &mut MysqlConnection, equip_id: i32, equip_type: &str, new_count: i32) -> bool {
    use crate::schema::Equipment::dsl::*;

    let target = Equipment
        .filter(id.eq(equip_id).and(equipment_type.eq(equip_type)));

    let result = diesel::update(target)
        .set(error_cnt.eq(new_count))
        .execute(conn);
    match result {
        Ok(affected) => affected == 1,
        Err(_) => false,
    }
}

pub fn update_red_correction_count(conn: &mut MysqlConnection, equip_id: i32, equip_type: &str, new_count: i32) -> bool {
    use crate::schema::Equipment::dsl::*;

    let target = Equipment
        .filter(id.eq(equip_id).and(equipment_type.eq(equip_type)));
    let result = diesel::update(target)
        .set(red_correction_cnt.eq(new_count))
        .execute(conn);
    match result {
        Ok(affected) => affected == 1,
        Err(_) => false,
    }
}

pub fn update_green_correction_count(conn: &mut MysqlConnection, equip_id: i32, equip_type: &str, new_count: i32) -> bool {
    use crate::schema::Equipment::dsl::*;

    let target = Equipment
        .filter(id.eq(equip_id).and(equipment_type.eq(equip_type)));
    let result = diesel::update(target)
        .set(green_correction_cnt.eq(new_count))
        .execute(conn);
    match result {
        Ok(affected) => affected == 1,
        Err(_) => false,
    }
}

pub fn get_ampere_data(conn: &mut MysqlConnection, equip_id: i32, equip_type: &str, is_red: bool) -> Vec<(f32, f32)> {
    use diesel::sql_query;
    use diesel::sql_types::Text;

    #[derive(QueryableByName)]
    struct AmpValue {
        #[diesel(sql_type = Text)]
        pub amp: String,
        #[diesel(sql_type = Text)]
        pub duty: String,
    }

    let query = format!(
        "SELECT SUBSTRING_INDEX(SUBSTRING_INDEX(rawData, '\n', {2}), '\n', -1) AS amp, \
        SUBSTRING_INDEX(SUBSTRING_INDEX(rawData, '\n', {3}), '\n', -1) AS duty
        FROM EquipmentStatus \
        WHERE \
            equipment_type like '{0}' AND equipment_id = {1} AND \
            SUBSTRING_INDEX(SUBSTRING_INDEX(rawData, '\n', {2}), '\n', -1) != '0' AND \
            abnormal = false \
        ORDER BY id DESC;
        ",
        equip_type,
        equip_id,
        if is_red { 3 } else { 4 },
        if is_red { 6 } else { 7 },
    );

    match sql_query(query).load::<AmpValue>(conn) {
        Ok(result) => {
            result.iter().map(|raw| {
                let ampere = raw.amp.clone().parse().unwrap_or(0f32);
                let duty = raw.duty.clone().parse().unwrap_or(0f32);
                (ampere, duty)
            }).collect()
        },
        Err(_) => {
            Vec::default()
        },
    }
}

pub fn get_amp_value(conn: &mut MysqlConnection, equip_id: i32, equip_type: &str, duty: &str, is_red: bool) -> String {
    use diesel::sql_query;
    use diesel::sql_types::Text;

    #[derive(QueryableByName)]
    struct AmpValue {
        #[diesel(sql_type = Text)]
        pub amp: String,
    }

    let query = format!(
        "SELECT SUBSTRING_INDEX(SUBSTRING_INDEX(rawData, '\n', {3}), '\n', -1) AS amp \
        FROM EquipmentStatus \
        WHERE \
            equipment_type like '{0}' AND equipment_id = {1} AND \
            SUBSTRING_INDEX(SUBSTRING_INDEX(rawData, '\n', {3}), '\n', -1) != '0' AND \
            SUBSTRING_INDEX(SUBSTRING_INDEX(rawData, '\n', {4}), '\n', -1) = '{2}' \
        ORDER BY id DESC \
        LIMIT 0, 1;
        ",
        equip_type,
        equip_id,
        duty,
        if is_red { 3 } else { 4 },
        if is_red { 6 } else { 7 },
    );

    match sql_query(query).load::<AmpValue>(conn) {
        Ok(result) => {
            if let Some(value) = result.first() {
                value.amp.clone()
            } else {
                String::new()
            }
        },
        Err(_) => {
            String::new()
        },
    }
}

pub fn get_all_equipment_status(conn: &mut MysqlConnection, equip_type: &str, equip_id: i32) -> Vec<(i32, String, String, bool, NaiveDateTime)> {
    use crate::schema::EquipmentStatus::dsl::*;

    let result = EquipmentStatus
        .filter(equipment_id.eq(equip_id).and(equipment_type.eq(equip_type)))
        .select((id, rawData, state, abnormal, receive_date))
        .order_by(receive_date.desc())
        .load::<(i32, String, String, bool, NaiveDateTime)>(conn);
    result.unwrap_or_default()
}

pub fn get_equipment_status(conn: &mut MysqlConnection, equip_id: i32, equip_type: &str) -> String {
    use crate::schema::EquipmentStatus::dsl::*;

    let result = EquipmentStatus
        .filter(equipment_id.eq(equip_id).and(equipment_type.eq(equip_type)))
        .select(rawData)
        .order_by(id.desc())
        .first::<String>(conn);
    result.unwrap_or("".to_string())
}

pub fn create_equipment_status(conn: &mut MysqlConnection, id: i32, equipment_type: &str, raw_data: &str, metric_state: EquipmentStatusState, abnormal: bool) {
    use crate::schema::EquipmentStatus;
    use chrono::Local;

    let current_time = Local::now().naive_local();
    let new_log = NewEquipmentStatus {
        equipment_id: id,
        equipment_type,
        raw_data,
        state: &metric_state.to_string(),
        abnormal,
        receive_date: current_time
    };
    let result = diesel::insert_into(EquipmentStatus::table)
        .values(&new_log)
        .execute(conn);
    match result {
        Ok(_) => {
            logger_log!("SQLService", format!("Save a {}-{:03} payload to database.", equipment_type, id));
        },
        Err(err) => println!("Error while saving equipment status \n{}", err),
    }
}




pub fn create_display_device_info(
    conn: &mut MysqlConnection,
    equipment_type: &str, 
    equip_id: i32,
    dataset_index: i32,
    led_g: i32,
    led_r: i32,
    cur_g: i32,
    cur_r: i32,
    cur_off_g: i32,
    cur_off_r: i32,
    temp: i32,
) {
    use crate::models::NewDisplayDeviceInfo;
    use crate::schema::DisplayDeviceInfo;   


    let new_record = NewDisplayDeviceInfo {
        id: dataset_index,
        equipment_type,
        equipment_id: equip_id,
        voltage_red: led_r,
        voltage_green: led_g,
        current_red: cur_r,
        current_green: cur_g,
        off_current_red: cur_off_r,
        off_current_green: cur_off_g,
        temperature: temp,
    };

    let result = diesel::insert_into(DisplayDeviceInfo::table)
        .values(&new_record)
        .execute(conn);

    match result {
        Ok(_) => logger_log!("SQLService", format!("Saved display data for {}-{}", equipment_type, equip_id)),
        Err(err) => println!("Error saving display device info: {err}"),
    }
}



use crate::models::Firedisplayinfo;
use chrono::Utc;
use uuid::Uuid;

pub async fn create_display_device_info_firebase(

    db_service: &DbService,
    equipment_type: String, 
        equip_id: i32,
        dataset_index: i32,
        led_g: i32,
        led_r: i32,
        cur_g: i32,
        cur_r: i32,
        cur_off_g: i32,
        cur_off_r: i32,
        temp: i32,
) {
    let info = Firedisplayinfo {
        id: Uuid::new_v4().to_string(),  // auto-generated Firestore doc ID
        deviceid: dataset_index,         // assigned to deviceid
        equipment_type,
        equipment_id: equip_id,
        voltage_green: led_g,
        voltage_red: led_r,
        current_green: cur_g,
        current_red: cur_r,
        off_current_green: cur_off_g,
        off_current_red: cur_off_r,
        temperature: temp,
        updated_at: Utc::now(),
    };

    match db_service.insert(info).await {
        Ok(_) => println!(" Firebase: Display device info saved"),
        Err(e) => eprintln!(" Firebase insert error: {:?}", e),
    }
}
