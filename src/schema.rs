use diesel::{allow_tables_to_appear_in_same_query, joinable};
diesel::table! {
    use diesel::sql_types::*;

    #[allow(non_snake_case)]
    Equipment (no) {
        no -> Integer,
        id -> Integer,
        #[max_length = 3]
        equipment_type -> Varchar,
        device_state -> Varchar,
        interval -> Integer,
        latitude -> Nullable<Double>,
        longitude -> Nullable<Double>,
        units -> Integer,
        error_cnt -> Integer,
        red_correction_cnt -> Integer,
        green_correction_cnt -> Integer,
        manufacturing_date -> Date,
        order_date -> Nullable<Date>,
        location_id -> Integer,
        order_company -> Nullable<Integer>,
        is_active -> Nullable<Bool>
    }
}

diesel::table! {
    use diesel::sql_types::*;

    #[allow(non_snake_case)]
    EquipmentLocation (id) {
        id -> Integer,
        name -> Text,
        latitude -> Nullable<Double>,
        longitude -> Nullable<Double>,
        manage_id -> Integer,
        install_company -> Integer,
        install_date -> Nullable<Datetime>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    #[allow(non_snake_case)]
    EquipmentStatus (id) {
        id -> Integer,
        #[max_length = 3]
        equipment_type -> Varchar,
        equipment_id -> Integer,
        rawData -> Text,
        state -> Varchar,
        abnormal -> Bool,
        receive_date -> Datetime
    }
}

diesel::table! {
    use diesel::sql_types::*;
    
    #[allow(non_snake_case)]
    DisplayDeviceInfo(no){
        no -> Integer,
        id -> Integer,
        equipment_type -> Varchar,
        equipment_id -> Integer ,
        voltage_red -> Integer,
        voltage_green -> Integer,
        current_red -> Integer,
        current_green -> Integer,
        off_current_red -> Integer,
        off_current_green -> Integer,
        temperature -> Integer
    }
}

joinable!(Equipment -> EquipmentLocation(location_id));
allow_tables_to_appear_in_same_query!(Equipment, EquipmentLocation, EquipmentStatus);