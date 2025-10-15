use chrono::{NaiveDateTime};
use crate::schema::{EquipmentStatus, DisplayDeviceInfo};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use diesel::{Insertable};

#[derive(Debug, Clone, Copy)]
pub enum EquipmentStatusState {
    Normal,
    Abnormal,
    Fault
}

impl fmt::Display for EquipmentStatusState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f, "{}",
            match self {
                EquipmentStatusState::Normal => "NORMAL",
                EquipmentStatusState::Abnormal => "ABNORMAL",
                EquipmentStatusState::Fault => "FAULT"
            }
        )
    }
}

impl FromStr for EquipmentStatusState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "NORMAL" => Ok(EquipmentStatusState::Normal),
            "ABNORMAL" => Ok(EquipmentStatusState::Abnormal),
            "FAULT" => Ok(EquipmentStatusState::Fault),
            _ => Ok(EquipmentStatusState::Normal)
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = EquipmentStatus)]
pub struct NewEquipmentStatus<'a> {
    pub equipment_type: &'a str,
    pub equipment_id: i32,
    #[diesel(column_name = "rawData")]
    pub raw_data: &'a str,
    pub state: &'a str,
    pub abnormal: bool,
    pub receive_date: NaiveDateTime,
}


#[derive(Insertable)]
#[diesel(table_name = DisplayDeviceInfo)]
pub struct NewDisplayDeviceInfo<'a> {
    pub id: i32,
    pub equipment_type: &'a str,
    pub equipment_id: i32,
    pub voltage_red: i32,
    pub voltage_green: i32,
    pub current_red: i32,
    pub current_green: i32,
    pub off_current_red: i32,
    pub off_current_green: i32,
    pub temperature: i32,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Firedisplayinfo {
    pub id: String,
    pub deviceid: i32,
    pub equipment_type: String,
    pub equipment_id: i32,
    pub voltage_red: i32,
    pub voltage_green: i32,
    pub current_red: i32,
    pub current_green: i32,
    pub off_current_red: i32,
    pub off_current_green: i32,
    pub temperature: i32,
    pub updated_at:DateTime<Utc>, 
}