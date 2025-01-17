use anyhow::Result;
use term_painter::{Attr::*, Color::*, ToStyle};

use crate::modules::*;
use display::border::Border;
use display::weathercode::WeatherCode;
use display::wind::Wind;
use regex::Regex;
use weather::Weather;

pub struct Current {
	title: String,
	temperature: String,
	apparent_temperature: String,
	humidity: String,
	dewpoint: String,
	wind: String,
	pressure: String,
	sun_time: String,
	wmo_code: WeatherCode,
	width: usize,
}

struct Dimensions {
	max_width: usize,
	min_width: usize,
	cell_width: usize,
}

impl Current {
	pub fn render(weather: &Weather, city: String) -> Result<()> {
		let dims = Dimensions {
			max_width: 60,
			min_width: 34,
			cell_width: 17,
		};

		let Current {
			title,
			temperature,
			apparent_temperature,
			humidity,
			dewpoint,
			wind,
			pressure,
			sun_time,
			wmo_code,
			width,
		} = Self::prepare(weather, city, &dims)?;

		// Border Top
		BrightBlack.with(|| {
			println!(
				"{}{}{} ",
				Border::TL.fmt(),
				Border::T.fmt().repeat(width),
				Border::TR.fmt()
			)
		});

		// Title
		println!(
			"{} {: ^3$} {}",
			BrightBlack.paint(Border::L.fmt()),
			// Bold.paint(title),
			title,
			BrightBlack.paint(Border::R.fmt()),
			width - 2
		);

		// Separator
		BrightBlack.with(|| println!("{}{}{}", Border::L.fmt(), "—".repeat(width), Border::R.fmt()));

		// Temperature
		println!(
			"{} {} {}{}{}",
			BrightBlack.paint(Border::L.fmt()),
			Bold.paint(&temperature),
			wmo_code.interpretation,
			" ".repeat(width - temperature.chars().count() - wmo_code.interpretation.chars().count() - 2),
			BrightBlack.paint(Border::R.fmt()),
		);
		println!(
			"{} {: <3$}{}",
			BrightBlack.paint(Border::L.fmt()),
			apparent_temperature,
			BrightBlack.paint(Border::R.fmt()),
			width - 1
		);

		// Blank line
		BrightBlack.with(|| println!("{}{}{}", Border::L.fmt(), " ".repeat(width), Border::R.fmt()));

		let humidity_dewpoint_row = format!("{: <2$}{}", humidity, dewpoint, dims.cell_width);
		println!(
			"{} {: <3$}{}",
			BrightBlack.paint(Border::L.fmt()),
			humidity_dewpoint_row,
			BrightBlack.paint(Border::R.fmt()),
			width - 1
		);

		let wind_pressure_row = format!("{: <2$}{}", wind, pressure, dims.cell_width);
		println!(
			"{} {: <3$}{}",
			BrightBlack.paint(Border::L.fmt()),
			wind_pressure_row,
			BrightBlack.paint(Border::R.fmt()),
			width - 1
		);

		// Sun times
		println!(
			"{} {: <3$}{}",
			BrightBlack.paint(Border::L.fmt()),
			sun_time,
			BrightBlack.paint(Border::R.fmt()),
			width - 1
		);

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

	fn prepare(weather: &Weather, city: String, dims: &Dimensions) -> Result<Self> {
		let title = Self::check_title_len(city, dims.max_width)?;
		let title_len = title.chars().count();
		let width = (if title_len > dims.min_width {
			title_len
		} else {
			dims.min_width
		}) + 3 * 2;

		let (current_hour, sunrise_hour, sunset_hour) = (
			weather.current_weather.time[11..13]
				.parse::<usize>()
				.unwrap_or_default(),
			weather.daily.sunrise[0][11..13].parse().unwrap_or_default(),
			weather.daily.sunset[0][11..13].parse().unwrap_or_default(),
		);
		let night = current_hour < sunrise_hour || current_hour > sunset_hour;
		let wmo_code = WeatherCode::resolve(&weather.current_weather.weathercode, Some(night))?;
		let wind_direction = Wind::get_direction(weather.current_weather.winddirection)?;

		let temperature = format!(
			"{} {}{}",
			wmo_code.icon, weather.current_weather.temperature, weather.hourly_units.temperature_2m
		);

		let apparent_temperature = format!(
			"Feels like {}{}",
			weather.hourly.apparent_temperature[current_hour], weather.hourly_units.temperature_2m
		);

		let humidity = format!(
			"Humidity: {}{}",
			weather.hourly.relativehumidity_2m[current_hour], weather.hourly_units.relativehumidity_2m,
		);
		let dewpoint = format!(
			"Dew Point: {}{}",
			weather.hourly.dewpoint_2m[current_hour], weather.hourly_units.dewpoint_2m
		);

		let wind = format!(
			"{} {}{} {}",
			wind_direction.icon,
			weather.current_weather.windspeed,
			weather.hourly_units.windspeed_10m,
			wind_direction.direction.fmt()
		);
		let pressure = format!(
			" {}{}",
			weather.hourly.surface_pressure[current_hour], weather.hourly_units.surface_pressure
		);

		let sun_time = format!(
			" {: <2$} {}",
			weather.daily.sunrise[0][11..16].to_string(),
			weather.daily.sunset[0][11..16].to_string(),
			dims.cell_width - 2
		);

		Ok(Current {
			title,
			temperature,
			apparent_temperature,
			humidity,
			dewpoint,
			wind,
			pressure,
			sun_time,
			wmo_code,
			width,
		})
	}

	fn check_title_len(title: String, max_width: usize) -> Result<String> {
		let title_len = title.chars().count();
		let mut new_title = if title_len > max_width {
			Self::trunc_title(title)?
		} else {
			title
		};
		if title_len > max_width {
			new_title = Self::check_title_len(new_title, max_width)?;
		}
		Ok(new_title)
	}

	fn trunc_title(title: String) -> Result<String> {
		// let title_commas = title.matches(',').count();
		// the results seems better for many places with overlong names when partially removing text
		// between first and second comma instead of removing it between penultimate and last comma

		let prep_re = format!("^((?:[^,]*,){{{}}})[^,]*,(.*)", 1);
		let re = Regex::new(&prep_re).unwrap();
		let truncated_title = re.replace(&title, "$1$2").to_string();

		Ok(truncated_title)
	}
}
