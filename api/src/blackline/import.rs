use crate::repo::device::DeviceRepo;
use crate::repo::gas_reading::{GasReading, GasReadingRepo};
use crate::repo::location_reading::{LocationReading, LocationReadingRepo};
use crate::warp_ext;
use crate::warp_ext::BoxReply;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceData {
    pub device: Device,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: u32,
    pub events: Vec<DeviceEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEvent {
    pub time: DateTime<Utc>,
    pub location_latitude: Option<String>,
    pub location_longitude: Option<String>,
    pub sensors: Option<Vec<SensorReading>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub r#type: String,
    pub value: f64,
}

#[derive(Clone)]
pub struct DeviceDataContext {
    pub device_repo: Arc<dyn DeviceRepo + Send + Sync + 'static>,
    pub gas_reading_repo: Arc<dyn GasReadingRepo + Send + Sync + 'static>,
    pub location_reading_repo: Arc<dyn LocationReadingRepo + Send + Sync + 'static>,
}

pub fn device_data_filter(context: DeviceDataContext) -> BoxedFilter<(Box<dyn Reply>,)> {
    warp::post()
        .and(warp::path("import"))
        .and(warp::path("deviceData"))
        .and(warp_ext::with_clone(context))
        .and(warp::body::json())
        .then(
            move |context: DeviceDataContext, device_data: DeviceData| async move {
                let device_id = device_data.device.id.to_string();
                if let Some(device) = context.device_repo.find_one(&device_id).await? {
                    let owner_id = device.owner_id;
                    let location_readings = device_data
                        .device
                        .events
                        .iter()
                        .map(|e| event_location_reading(e, &owner_id))
                        .filter_map(|e| e.transpose())
                        .collect::<anyhow::Result<Vec<LocationReading>>>()?;
                    context
                        .location_reading_repo
                        .insert_many(location_readings)
                        .await?;
                    let gas_readings = device_data
                        .device
                        .events
                        .iter()
                        .map(|e| event_gas_readings(e, &owner_id))
                        .map(|r| match r {
                            Ok(v) => v.into_iter().map(Ok).collect::<Vec<_>>(),
                            Err(e) => vec![Err(e)],
                        })
                        .flatten()
                        .collect::<anyhow::Result<Vec<GasReading>>>()?;
                    context.gas_reading_repo.insert_many(gas_readings).await?;
                    Ok(warp::reply().boxed())
                } else {
                    Err(anyhow::anyhow!("device not found"))
                }
            },
        )
        .map(warp_ext::convert_err)
        .boxed()
}

fn event_location_reading(
    event: &DeviceEvent,
    person_id: &str,
) -> anyhow::Result<Option<LocationReading>> {
    if let Some(longitude) = &event.location_longitude {
        if let Some(latitude) = &event.location_latitude {
            return Ok(Some(LocationReading {
                timestamp: event.time,
                person_id: person_id.to_string(),
                coordinates: vec![longitude.parse()?, latitude.parse()?],
            }));
        }
    }
    Ok(None)
}

fn event_gas_readings(event: &DeviceEvent, person_id: &str) -> anyhow::Result<Vec<GasReading>> {
    if let Some(longitude) = &event.location_longitude {
        if let Some(latitude) = &event.location_latitude {
            if let Some(sensors) = &event.sensors {
                return sensors
                    .iter()
                    .map(|s| {
                        Ok(GasReading {
                            timestamp: event.time,
                            person_id: person_id.to_string(),
                            gas: s.r#type.to_string(),
                            density: s.value,
                            coordinates: vec![longitude.parse()?, latitude.parse()?],
                        })
                    })
                    .collect::<anyhow::Result<_>>();
            }
        }
    }
    Ok(Vec::new())
}
