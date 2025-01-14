use anyhow::Result;
use chrono::offset::TimeZone;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use term_painter::{Color::*, ToStyle};

use crate::modules::display::border::Border;
use crate::modules::display::weathercode::WeatherCode;
use crate::modules::weather::Weather;

#[derive(Serialize, Deserialize, Debug)]
pub struct Forecast {
	pub days: Vec<ForecastDay>,
	pub width: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForecastDay {
	pub date: String,
	pub weather: String,
	pub interpretation: String,
	// pub icon: String,
}

impl Forecast {
	pub fn render_forecast(data: &Weather) -> Result<()> {
		let forecast = Forecast::generate_days(data)?;
		let width = forecast.width + 8;

		// Border Top
		BrightBlack.with(|| {
			println!(
				"{}{}{} ",
				Border::TL.fmt(),
				Border::T.fmt().repeat(width),
				Border::TR.fmt()
			)
		});

		let mut chunks = forecast.days.chunks(1).peekable();

		let mut n = 0;
		while let Some(_) = chunks.next() {
			let merge = format!(
				"{}      {}{}{}",
				forecast.days[n].date,
				forecast.days[n].weather,
				" ".repeat(
					width
						- forecast.days[n].date.len()
						- forecast.days[n].weather.len()
						- forecast.days[n].interpretation.len()
						- 4
				),
				forecast.days[n].interpretation
			);
			println!(
				"{} {: <3$}{}",
				BrightBlack.paint(Border::L.fmt()),
				merge,
				BrightBlack.paint(Border::R.fmt()),
				width - 1
			);
			if chunks.peek().is_some() {
				// Separator
				BrightBlack.with(|| println!("{}{}{}", Border::L.fmt(), "—".repeat(width), Border::R.fmt()));
			}

			n += 1;
		}

		// Border Bottom
		BrightBlack.with(|| {
			println!(
				"{}{}{}",
				Border::BL.fmt(),
				Border::B.fmt().repeat(width),
				Border::BR.fmt()
			)
		});
		Ok(())
	}

	fn generate_days(data: &Weather) -> Result<Self> {
		let mut days = Vec::new();
		let mut width: usize = 0;

		for (i, _) in data.daily.time.iter().enumerate() {
			let time = &data.daily.time[i];
			let date = Utc
				.ymd(
					time[0..4].parse().unwrap_or_default(),
					time[5..7].parse().unwrap_or_default(),
					time[8..10].parse().unwrap_or_default(),
				)
				.and_hms(0, 0, 0);
			// let date = date.format("%a, %b %e").to_string();
			let date = &date.to_rfc2822()[..11];

			let weather_code = WeatherCode::resolve(&data.daily.weathercode[i], None)?;
			let weather = format!(
				"{} {}{}/{}{}",
				weather_code.icon,
				data.daily.temperature_2m_max[i],
				data.daily_units.temperature_2m_max,
				data.daily.temperature_2m_min[i],
				data.daily_units.temperature_2m_min,
			);
			let merge = format!("{}{}{}", date, weather, weather_code.interpretation);
			if merge.len() > width {
				width = merge.len();
			}

			let day: ForecastDay = {
				ForecastDay {
					date: date.to_string(),
					weather,
					interpretation: weather_code.interpretation,
					// icon: weather_code.icon.unwrap_or_default(),
				}
			};

			days.push(day);
		}

		Ok(Forecast { width, days })
	}
}
