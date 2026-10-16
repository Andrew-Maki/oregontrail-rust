//! All console input / output helpers.

use std::io::{self, Write};

pub const WIDTH: usize = 72;

// ---------------------------------------------------------------- output ---

pub fn flush() {
    let _ = io::stdout().flush();
}

pub fn rule() {
    println!("{}", "-".repeat(WIDTH));
}

pub fn thick_rule() {
    println!("{}", "=".repeat(WIDTH));
}

pub fn blank() {
    println!();
}

/// A centred line of text.
pub fn center(text: &str) {
    let len = text.chars().count();
    if len >= WIDTH {
        println!("{}", text);
    } else {
        println!("{}{}", " ".repeat((WIDTH - len) / 2), text);
    }
}

/// A boxed heading, e.g. a landmark banner.
pub fn banner(title: &str) {
    blank();
    thick_rule();
    center(&title.to_uppercase());
    thick_rule();
}

pub fn heading(title: &str) {
    blank();
    println!("{}", title);
    rule();
}

/// Word-wrap `text` to the console width and print it.
pub fn wrap(text: &str) {
    wrap_indent(text, 0);
}

pub fn wrap_indent(text: &str, indent: usize) {
    let pad = " ".repeat(indent);
    let limit = WIDTH.saturating_sub(indent).max(20);
    for paragraph in text.split('\n') {
        if paragraph.trim().is_empty() {
            println!();
            continue;
        }
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if line.is_empty() {
                line.push_str(word);
            } else if line.chars().count() + 1 + word.chars().count() <= limit {
                line.push(' ');
                line.push_str(word);
            } else {
                println!("{}{}", pad, line);
                line.clear();
                line.push_str(word);
            }
        }
        if !line.is_empty() {
            println!("{}{}", pad, line);
        }
    }
}

/// A tombstone, drawn when somebody dies.
pub fn tombstone(lines: &[String]) {
    const INNER: usize = 32;
    blank();
    println!("              .-\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"\"-.");
    println!("            .'                          '.");
    println!("           /            R. I. P.          \\");
    println!("          ;{};", " ".repeat(INNER));
    for i in 0..4 {
        let text: String = lines
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("")
            .chars()
            .take(INNER - 2)
            .collect();
        let len = text.chars().count();
        let left = (INNER - len) / 2;
        let right = INNER - len - left;
        println!(
            "          |{}{}{}|",
            " ".repeat(left),
            text,
            " ".repeat(right)
        );
    }
    println!("          |{}|", " ".repeat(INNER));
    println!("        __|{}|__", "_".repeat(INNER));
    println!("       /{}\\", " ".repeat(INNER + 6));
    blank();
}

// ----------------------------------------------------------------- input ---

/// Read a line of input. Treats EOF as "quit" so the game behaves sanely when
/// driven from a pipe or a script.
pub fn read_line() -> String {
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(0) => {
            blank();
            println!("(end of input -- goodbye)");
            std::process::exit(0);
        }
        Ok(_) => buf.trim().to_string(),
        Err(_) => {
            blank();
            std::process::exit(0);
        }
    }
}

pub fn prompt(message: &str) -> String {
    print!("{}", message);
    flush();
    read_line()
}

pub fn pause() {
    print!("\nPress ENTER to continue. ");
    flush();
    let _ = read_line();
}

/// Numbered menu. Returns a 1-based choice.
pub fn menu(title: &str, options: &[&str]) -> usize {
    loop {
        if !title.is_empty() {
            heading(title);
        }
        for (i, opt) in options.iter().enumerate() {
            println!("   {:>2}. {}", i + 1, opt);
        }
        blank();
        let raw = prompt("What is your choice? ");
        if let Ok(n) = raw.parse::<usize>() {
            if n >= 1 && n <= options.len() {
                return n;
            }
        }
        println!("\nPlease enter a number from 1 to {}.", options.len());
    }
}

pub fn yes_no(question: &str) -> bool {
    loop {
        let raw = prompt(&format!("{} (y/n) ", question)).to_lowercase();
        match raw.as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => println!("Please answer 'y' or 'n'."),
        }
    }
}

/// Ask for an integer in `lo..=hi`. Empty input yields `default` if given.
pub fn prompt_int(message: &str, lo: i64, hi: i64, default: Option<i64>) -> i64 {
    loop {
        let raw = prompt(message);
        if raw.is_empty() {
            if let Some(d) = default {
                return d;
            }
        }
        match raw.parse::<i64>() {
            Ok(n) if n >= lo && n <= hi => return n,
            Ok(_) => println!("Please enter a number from {} to {}.", lo, hi),
            Err(_) => println!("Please enter a whole number from {} to {}.", lo, hi),
        }
    }
}

/// Ask for a non-empty name.
pub fn prompt_name(message: &str) -> String {
    loop {
        let raw = prompt(message);
        let cleaned: String = raw
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '\'' || *c == '.')
            .collect();
        let cleaned = cleaned.trim();
        if !cleaned.is_empty() {
            return cleaned.chars().take(14).collect();
        }
        println!("Please type a name.");
    }
}

// ------------------------------------------------------------ formatting ---

/// Format cents as `$1,234.56`.
pub fn money(cents: i64) -> String {
    let neg = cents < 0;
    let abs = cents.abs();
    let dollars = abs / 100;
    let rem = abs % 100;
    format!(
        "{}${}.{:02}",
        if neg { "-" } else { "" },
        group(dollars),
        rem
    )
}

/// Insert thousands separators.
pub fn group(n: i64) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

pub fn plural(n: u32, singular: &str, plural_form: &str) -> String {
    if n == 1 {
        format!("{} {}", n, singular)
    } else {
        format!("{} {}", group(i64::from(n)), plural_form)
    }
}

/// Join a list naturally: "a, b and c".
pub fn list_and(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        2 => format!("{} and {}", items[0], items[1]),
        _ => {
            let head = items[..items.len() - 1].join(", ");
            format!("{} and {}", head, items[items.len() - 1])
        }
    }
}
