use mysql::*;
use mysql::prelude::*;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use structopt::StructOpt;
use prettytable::{Table, Row as PrettyRow, Cell, format};
use std::error::Error;
use std::path::PathBuf;
use dirs::home_dir;
use colored::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "mysql", about = "Cross-platform MySQL client")]
struct Opts {
    /// Host to connect to
    #[structopt(short, long, default_value = "localhost")]
    host: String,

    /// Port number to connect to
    #[structopt(short = "P", long, default_value = "3306")]
    port: u16,

    /// Username for login
    #[structopt(short = "u", long)]
    user: Option<String>,

    /// Password for login
    #[structopt(short = "p", long)]
    password: Option<String>,

    /// Database to use
    #[structopt(short = "D", long)]
    database: Option<String>,

    /// Execute command and quit
    #[structopt(short = "e", long)]
    execute: Option<String>,

    /// Disable colors in output
    #[structopt(long)]
    no_colors: bool,
}

struct MySQLClient {
    conn: Conn,
    current_db: Option<String>,
    use_colors: bool,
    host: String,
    port: u16,
}

impl MySQLClient {
    fn new(opts: &Opts) -> Result<Self, Box<dyn Error>> {
        let builder = OptsBuilder::new()
            .user(opts.user.as_deref())
            .pass(opts.password.as_deref())
            .ip_or_hostname(Some(opts.host.as_str()))
            .tcp_port(opts.port)
            .db_name(opts.database.as_deref());

        let conn = Conn::new(builder)?;
        let current_db = opts.database.clone();
        let use_colors = !opts.no_colors;
        let host = opts.host.clone();
        let port = opts.port;

        Ok(MySQLClient { conn, current_db, use_colors, host, port })
    }

    fn format_cell(&self, value: String, is_null: bool) -> String {
        if !self.use_colors {
            return if is_null { "NULL".to_string() } else { value };
        }

        if is_null {
            "NULL".bright_red().to_string()
        } else {
            value.bright_white().to_string()
        }
    }

    fn execute_query(&mut self, query: &str) -> Result<Option<QueryResult>, Box<dyn Error>> {
        // Handle special commands
        match query.trim().to_lowercase().as_str() {
            "status" => return self.show_status(),
            "clear" | "\\c" => {
                print!("\x1B[2J\x1B[1;1H");  // Clear screen
                return Ok(None);
            }
            _ => {}
        }
    
        let start_time = std::time::Instant::now();
        let use_colors = self.use_colors;
    
        // Handle USE command
        if query.trim().to_lowercase().starts_with("use ") {
            let db = query.trim()[4..].trim().trim_matches(';');
            self.conn.select_db(db)?;
            self.current_db = Some(db.to_string());
            
            let msg = format!("Database changed to '{}'", db);
            println!("{}", if use_colors { msg.green().to_string() } else { msg });
            
            return Ok(None);
        }
    
        // Execute the query
        let affected_rows = self.conn.affected_rows();
        let result = self.conn.query_iter(query)?;
        let column_info = result.columns().as_ref().to_vec();
    
        if column_info.is_empty() {
            // Handle non-SELECT queries
            let elapsed = start_time.elapsed();
            
            if affected_rows > 0 {
                let msg = format!(
                    "Query OK, {} {} affected ({:.2} sec)",
                    affected_rows,
                    if affected_rows == 1 { "row" } else { "rows" },
                    elapsed.as_secs_f64()
                );
                println!("{}", if use_colors { msg.green().to_string() } else { msg });
            }
            return Ok(None);
        }
    
        // Format SELECT query results
        let mut table = Table::new();
        let format = format::FormatBuilder::new()
            .column_separator('│')
            .borders('│')
            .separator(format::LinePosition::Top, format::LineSeparator::new('─', '┌', '┐', '┬'))
            .separator(format::LinePosition::Bottom, format::LineSeparator::new('─', '└', '┘', '┴'))
            .separator(format::LinePosition::Title, format::LineSeparator::new('─', '├', '┤', '┼'))
            .padding(1, 1)
            .build();
        table.set_format(format);
    
        // Add header row
        let headers: Vec<Cell> = column_info.iter()
            .map(|c| {
                let header = if use_colors {
                    c.name_str().bright_cyan().to_string()
                } else {
                    c.name_str().to_string()
                };
                Cell::new(&header).style_spec("b")
            })
            .collect();
        table.add_row(PrettyRow::new(headers));
    
        // Calculate maximum widths for each column
        let mut max_widths: Vec<usize> = column_info.iter()
            .map(|c| c.name_str().len())
            .collect();
    
        // Collect all rows
        let rows: Vec<mysql::Row> = result.collect::<Result<Vec<_>, _>>()?;
    
        // First pass to find maximum widths
        for row in &rows {
            for i in 0..column_info.len() {
                if i < max_widths.len() {
                    let formatted = match row.get_opt(i) {
                        Some(val) => {
                            match val {
                                Ok(Value::NULL) => "NULL".to_string(),
                                Ok(Value::Bytes(bytes)) => String::from_utf8_lossy(&bytes).into_owned(),
                                Ok(Value::Int(n)) => n.to_string(),
                                Ok(Value::UInt(n)) => n.to_string(),
                                Ok(Value::Float(f)) => f.to_string(),
                                Ok(Value::Double(d)) => d.to_string(),
                                Ok(Value::Date(y, m, d, h, i, s, _)) => 
                                    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, i, s),
                                Ok(Value::Time(neg, d, h, i, s, _)) => {
                                    let sign = if neg { "-" } else { "" };
                                    format!("{}{}.{:02}:{:02}:{:02}", sign, d, h, i, s)
                                },
                                Err(_) => "ERROR".to_string()
                            }
                        },
                        _ => "NULL".to_string()
                    };
                    max_widths[i] = max_widths[i].max(formatted.len());
                }
            }
        }
    
        // Add data rows with proper width alignment
        for row in rows {
            let cells: Vec<Cell> = (0..column_info.len())
                .map(|i| {
                    let val = row.get_opt(i);
                    let (value, is_null) = match val {
                        Some(Ok(val)) => {
                            let formatted = match val {
                                Value::NULL => ("NULL".to_string(), true),
                                Value::Bytes(bytes) => (String::from_utf8_lossy(&bytes).into_owned(), false),
                                Value::Int(n) => (n.to_string(), false),
                                Value::UInt(n) => (n.to_string(), false),
                                Value::Float(f) => (f.to_string(), false),
                                Value::Double(d) => (d.to_string(), false),
                                Value::Date(y, m, d, h, i, s, _) => 
                                    (format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h, i, s), false),
                                Value::Time(neg, d, h, i, s, _) => {
                                    let sign = if neg { "-" } else { "" };
                                    (format!("{}{}.{:02}:{:02}:{:02}", sign, d, h, i, s), false)
                                }
                            };
                            formatted
                        },
                        _ => ("NULL".to_string(), true)
                    };
                
                    let formatted = if use_colors {
                        if is_null {
                            "NULL".bright_red().to_string()
                        } else {
                            value.bright_white().to_string()
                        }
                    } else {
                        if is_null { "NULL".to_string() } else { value }
                    };
                
                    Cell::new(&formatted)
                })
                .collect();
            table.add_row(PrettyRow::new(cells));
        }
    
        let row_count = table.len() - 1; // Subtract 1 to account for header row
        let elapsed = start_time.elapsed();
        let summary = format!(
            "{} {} in set ({:.2} sec)",
            row_count,
            if row_count == 1 { "row" } else { "rows" },
            elapsed.as_secs_f64()
        );
    
        Ok(Some(QueryResult { table, summary }))
    }

    fn show_status(&mut self) -> Result<Option<QueryResult>, Box<dyn Error>> {
        let mut table = Table::new();
        let format = format::FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .padding(1, 1)
            .build();
        table.set_format(format);

        // Server info
        let server_version: String = self.conn.query_first("SELECT VERSION()")?.unwrap_or_default();
        table.add_row(PrettyRow::new(vec![
            Cell::new("Server version:").style_spec("Fb"),
            Cell::new(&server_version),
        ]));

        // Connection info
        table.add_row(PrettyRow::new(vec![
            Cell::new("Server:").style_spec("Fb"),
            Cell::new(&format!("{}:{}", self.host, self.port)),
        ]));

        // Database info
        table.add_row(PrettyRow::new(vec![
            Cell::new("Current database:").style_spec("Fb"),
            Cell::new(self.current_db.as_deref().unwrap_or("None")),
        ]));

        // Character set info
        let charset: String = self.conn.query_first("SELECT @@character_set_client")?.unwrap_or_default();
        table.add_row(PrettyRow::new(vec![
            Cell::new("Character set:").style_spec("Fb"),
            Cell::new(&charset),
        ]));

        Ok(Some(QueryResult { 
            table,
            summary: String::new()
        }))
    }
}

struct QueryResult {
    table: Table,
    summary: String,
}

fn print_welcome_message(client: &mut MySQLClient) {
    if let Ok(Some(version)) = client.conn.query_first::<String, _>("SELECT VERSION()") {
        let banner = format!(r#"
Welcome to the MySQL monitor.  Commands end with ;

Server version: {}
Connection Id: {}

Copyright (c) 2000, 2024, Oracle and/or its affiliates.
Rust MySQL Monitor. A cross-platform MySQL client.

Type 'help;' or '\h' for help. Type '\c' to clear the current input statement.
"#, version, client.conn.connection_id());

        if client.use_colors {
            println!("{}", banner.bright_blue());
        } else {
            println!("{}", banner);
        }
    }
}

fn format_prompt(client: &MySQLClient, is_continuation: bool) -> String {
    if is_continuation {
        if client.use_colors {
            "    -> ".bright_green().to_string()
        } else {
            "    -> ".to_string()
        }
    } else {
        let db_str = client.current_db
            .as_ref()
            .map(|db| format!("({})", db))
            .unwrap_or_default();
        
        if client.use_colors {
            format!("mysql{} > ", db_str).bright_green().to_string()
        } else {
            format!("mysql{} > ", db_str)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();
    let mut client = MySQLClient::new(&opts)?;

    // Handle -e execute flag
    if let Some(query) = opts.execute {
        if let Some(result) = client.execute_query(&query)? {
            result.table.printstd();
            if !result.summary.is_empty() {
                println!("\n{}", if client.use_colors {
                    result.summary.green().to_string()
                } else {
                    result.summary
                });
            }
        }
        return Ok(());
    }

    // Set up interactive mode
    let history_file = home_dir()
        .map(|mut path| {
            path.push(".mysql_history");
            path
        })
        .unwrap_or_else(|| PathBuf::from(".mysql_history"));

    let mut rl = Editor::<(), FileHistory>::new()?;
    if rl.load_history(&history_file).is_err() {
        println!("No previous history.");
    }

    print_welcome_message(&mut client);

    let mut query_buffer = String::new();
    loop {
        let prompt = format_prompt(&client, !query_buffer.is_empty());

        match rl.readline(&prompt) {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                
                query_buffer.push_str(&line);
                query_buffer.push(' ');

                if line.trim().ends_with(';') {
                    match client.execute_query(&query_buffer) {
                        Ok(Some(result)) => {
                            result.table.printstd();
                            if !result.summary.is_empty() {
                                println!("\n{}", if client.use_colors {
                                    result.summary.green().to_string()
                                } else {
                                    result.summary
                                });
                            }
                        }
                        Ok(None) => {}
                        Err(e) => eprintln!("{}", if client.use_colors {
                            format!("Error: {}", e).bright_red().to_string()
                        } else {
                            format!("Error: {}", e)
                        }),
                    }
                    query_buffer.clear();
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                query_buffer.clear();
            }
            Err(ReadlineError::Eof) => {
                println!("Bye");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history(&history_file)?;
    Ok(())
}