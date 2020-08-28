use reqwest::blocking::Client;

use std::collections::HashMap;
use std::io::Cursor;
use std::time::Instant;

/// Data structure type of the data to be sent in the request form.
#[derive(Clone, Copy, Debug)]
enum FormDataType {
	None,
	DuckStart,
}

/// Holds data to be sent in the request form.
#[derive(Debug, Default, serde::Serialize)]
struct FormData {
	q: String
}

impl FormData {
	/// Embeds query text into the form in an appropriate manner.
	fn from_query(form: FormDataType, query: &str) -> FormData {
		match form {
			FormDataType::None => Default::default(),
			FormDataType::DuckStart => FormData { q: query.into(), ..Default::default() },
		}
	}
}

/// Information about a search source.
#[derive(Clone)]
struct SourceData<'a, E: std::error::Error> {
	req_method: reqwest::Method,
	base_url: String,
	form: FormDataType,
	term_process: Option<&'a dyn Fn(&str) -> String>,
	text_process: &'a dyn Fn(reqwest::blocking::Response) -> Result<String, E>,
}

/// Helper for `main`.
fn error_exit(msg: &str, code: i32) -> ! {
	eprintln!("{}", msg);
	std::process::exit(code)
}

/// A basic function for displaying search results.
///
/// Use this as `text_process` in `SourceData` if you have no other
/// suitable function.
fn default_result_disp(resp: reqwest::blocking::Response) -> Result<String, reqwest::Error> {
	Ok(
		html2text::from_read(
			Cursor::new(resp.text()?),
			std::env::var("COLUMNS")
				.map(|s| s.parse::<usize>())
				.unwrap_or(Ok(80))
				.unwrap_or(80)
		)
		.replace('─', "-")
		.replace('│', "|")
		.replace('┼', "+")
		.replace('�', "<?>")
	)
}

/// Runs `tsurch`.
fn main() {
	let mut canonical = HashMap::new();
	{
		let mut f = Vec::new();

		// Aliases
		f.push(("ddg", "duckduckgo"));
		f.push(("duck", "duckduckgo"));
		f.push(("wiki", "wikipedia"));
		f.push(("wp", "wikipedia"));
		f.push(("sp", "startpage"));
		f.push(("start", "startpage"));
		f.push(("rs", "rustdoc"));
		f.push(("rdoc", "rustdoc"));

		canonical.extend(f.into_iter());
	}

	let mut source_data = HashMap::new();
	{
		let mut d = Vec::new();

		d.push(("duckduckgo", SourceData {
			req_method: reqwest::Method::POST,
			base_url: "https://lite.duckduckgo.com/lite".into(),
			form: FormDataType::DuckStart,
			term_process: None,
			text_process: &default_result_disp,
		}));
		d.push(("startpage", SourceData {
			req_method: reqwest::Method::POST,
			base_url: "https://startpage.com/sp/search".into(),
			form: FormDataType::DuckStart,
			term_process: None,
			text_process: &default_result_disp,
		}));
		d.push(("wikipedia", SourceData {
			req_method: reqwest::Method::GET,
			base_url: "https://en.wikipedia.org/w/index.php?action=render&title=".into(),
			form: FormDataType::None,
			term_process: None,
			text_process: &default_result_disp,
		}));
		d.push(("rustdoc", SourceData {
			req_method: reqwest::Method::GET,
			base_url: "https://doc.rust-lang.org/".into(),
			form: FormDataType::None,
			term_process: None,
			text_process: &default_result_disp,
		}));

		source_data.extend(d.into_iter());
	}

	let clap_matches = clap::App::new("tsurch")
		.version("0.1.0")
		.author("vkcz")
		.about("Command-line web search tool")
		.arg_from_usage("-s, --source=[SOURCE] 'Select search result source (defaults to `ddg`)'")
		.arg_from_usage("<TERM> 'Search term(s)'")
		.get_matches();

	let search_source_raw = clap_matches.value_of("source").unwrap_or("ddg");
	let search_source_canon = canonical.get(search_source_raw).unwrap_or(&search_source_raw);
	let technique = match source_data.get(search_source_canon) {
		Some(t) => t,
		None => error_exit(&format!("Invalid search engine \"{}\"", search_source_raw), 2)
	};
	let search_term_raw = clap_matches.value_of("TERM").unwrap();
	let search_term_canon = match technique.term_process {
		Some(ref f) => (f)(search_term_raw),
		None => search_term_raw.into()
	};

	println!("Searching on {} for \"{}\"", search_source_canon, search_term_raw);

	let start_time = Instant::now();
	let req_url = if technique.req_method == reqwest::Method::GET {
		technique.base_url.clone() + &search_term_canon
	} else {
		technique.base_url.clone()
	};
	let client = Client::new();
	let resp = client.request(technique.req_method.clone(), &req_url)
		// obviously fake User-Agent
		.header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:80.0) Gecko/20100101 Firefox/80.0")
		.form(&FormData::from_query(technique.form, &search_term_canon))
		.send();

	match resp {
		Ok(resp) => if resp.status().is_success() {
			match (technique.text_process)(resp) {
				Ok(t) => println!(
					"({} seconds)\n{}",
					(Instant::now() - start_time).as_millis() as f64 / 1000.,
					t
				),
				Err(_) => error_exit("Failed to construct results text", 5)
			}
		} else {
			error_exit(&format!("Non-successful response ({})", resp.status().as_u16()), 4)
		},
		Err(_) => error_exit("Request failed", 3)
	}
}
