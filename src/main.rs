use plotters::prelude::*;
use chrono::{DateTime, Utc};
use serde_json::{Value};
use std::fs::File;
use std::io::{self, BufRead};

fn moving_average(data: &[f64], window_size: usize) -> Vec<f64> {
    let mut result = Vec::new();
    for i in 0..=data.len() - window_size {
        let sum: f64 = data[i..i + window_size].iter().sum();
        result.push(sum / window_size as f64);
    }
    result
}
  fn get_min_f64(values: &[f64]) -> f64 {
      values.iter()
          .fold(f64::INFINITY, |a, &b| a.min(b))
  }

  fn get_max_f64(values: &[f64]) -> f64 {
      values.iter()
          .fold(f64::NEG_INFINITY, |a, &b| a.max(b))
  }

  fn main() -> Result<(), Box<dyn std::error::Error>> {
      let mut timestamps: Vec<DateTime<Utc>> = Vec::new();
      let mut current_altitudes: Vec<f64> = Vec::new();
      let mut zero_altitudes: Vec<f64> = Vec::new();
      let mut relative_altitudes: Vec<f64> = Vec::new();
      let mut speeds: Vec<f64> = Vec::new();
      let mut speed_alarms: Vec<i32> = Vec::new();

      // Read JSON data
      let file = File::open("flight_log.json")?;
      let reader = io::BufReader::new(file);
    
      for line in reader.lines() {
          let data: Value = serde_json::from_str(&line?)?;
          let timestamp = if let Some(ts_str) = data["timestamp"].as_str() {
              // Try parsing with explicit timezone offset
              DateTime::parse_from_str(
                  &format!("{} +0000", ts_str),
                  "%Y-%m-%d %H:%M:%S.%f %z"
              )?.with_timezone(&Utc)
          } else {
              return Err("Missing timestamp field".into());
          };
        
          timestamps.push(timestamp);
                let current_altitude = data["message"]["Altitude"]["current"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or("Missing or invalid current altitude")?;

                let zero_altitude = data["message"]["Altitude"]["zero"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or("Missing or invalid zero altitude")?;
                
                let relative_altitude = data["message"]["Altitude"]["delta"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .ok_or("Missing or invalid relative altitude")?;

                let speed = data["message"]["Flight"]["speed"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or("Missing or invalid speed")?;

                let speed_alarm = data["message"]["Flight"]["speedAlarm"]
                    .as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or("Missing or invalid speed alarm")?;

                //timestamps.push(timestamp);
                current_altitudes.push(current_altitude);
                zero_altitudes.push(zero_altitude);
                relative_altitudes.push(relative_altitude);
                speeds.push(speed);
                speed_alarms.push(speed_alarm as i32);
            }

      let window_size = 5;
      let smoothed_current = moving_average(&current_altitudes, window_size);
      let smoothed_zero = moving_average(&zero_altitudes, window_size);
      let smoothed_relative = moving_average(&relative_altitudes, window_size);
      let smoothed_speeds = moving_average(&speeds, window_size);

      let root = BitMapBackend::new("flight_data_plot_rust.png", (1200, 1200))
          .into_drawing_area();
      root.fill(&WHITE)?;

      let areas = root.split_evenly((4, 1));
        // Plot 1: Altitude
        let mut chart = ChartBuilder::on(&areas[0])
            .caption("Altitude (Current and Zero Reference)", ("sans-serif", 20))
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(
                timestamps[0]..timestamps[timestamps.len() - 1], 
                get_min_f64(&current_altitudes)..get_max_f64(&current_altitudes)
            )?;

        chart.configure_mesh().draw()?;

        chart.draw_series(LineSeries::new(
            timestamps[window_size-1..].iter().zip(smoothed_current.iter()).map(|(x, y)| (*x, *y)),
            &BLUE,
        ))?;

        chart.draw_series(LineSeries::new(
            timestamps[window_size-1..].iter().zip(smoothed_zero.iter()).map(|(x, y)| (*x, *y)),
            &RED,
        ))?;

        // Plot 2: Relative Altitude
         let mut chart = ChartBuilder::on(&areas[1])
            .caption("Relative Altitude", ("sans-serif", 20))
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(
                timestamps[0]..timestamps[timestamps.len() - 1],
                get_min_f64(&relative_altitudes)..get_max_f64(&relative_altitudes)
            )?;
            

        chart.configure_mesh().draw()?;

        chart.draw_series(LineSeries::new(
            timestamps[window_size-1..].iter().zip(smoothed_relative.iter()).map(|(x, y)| (*x, *y)),
            &GREEN,
        ))?;

        // Plot 3: Speed
        let mut chart = ChartBuilder::on(&areas[2])
            .caption("Speed", ("sans-serif", 20))
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(
                timestamps[0]..timestamps[timestamps.len() - 1],
                get_min_f64(&speeds)..get_max_f64(&speeds)
            )?;

        chart.configure_mesh().draw()?;

        chart.draw_series(LineSeries::new(
            timestamps[window_size-1..].iter().zip(smoothed_speeds.iter()).map(|(x, y)| (*x, *y)),
            &BLUE,
        ))?;

        // Plot 4: Speed Alarms
        let mut chart = ChartBuilder::on(&areas[3])
            .caption("Speed Alarms", ("sans-serif", 20))
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 40)
            .build_cartesian_2d(
                timestamps[0]..timestamps[timestamps.len() - 1],
                0f64..2f64
            )?;

        chart.configure_mesh().draw()?;

        chart.draw_series(
            speed_alarms.iter()
                .zip(timestamps.iter())
                .filter(|(alarm, _)| **alarm == 1)
                .map(|(_, t)| Circle::new((*t, 1.0), 3, &RED))
        )?;

      Ok(())
  }