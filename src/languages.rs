use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref EXTENSIONS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("rs", "Rust");
        m.insert("go", "Go");
        m.insert("js", "JavaScript");
        m.insert("py", "Python");
        m.insert("java", "Java");
        m.insert("c", "C");
        m.insert("cpp", "C++");
        m.insert("h", "C/C++ Header");
        m.insert("hpp", "C++ Header");
        m.insert("cs", "C#");
        m.insert("html", "HTML");
        m.insert("css", "CSS");
        m.insert("md", "Markdown");
        m.insert("json", "JSON");
        m.insert("yml", "YAML");
        m.insert("yaml", "YAML");
        m.insert("xml", "XML");
        m.insert("sh", "Shell Script");
        m.insert("bat", "Batch Script");
        m.insert("ps1", "PowerShell Script");
        m.insert("sql", "SQL");
        m.insert("rb", "Ruby");
        m.insert("php", "PHP");
        m.insert("swift", "Swift");
        m.insert("kt", "Kotlin");
        m.insert("ts", "TypeScript");
        m.insert("scala", "Scala");
        m.insert("lua", "Lua");
        m.insert("pl", "Perl");
        m.insert("r", "R");
        m.insert("dart", "Dart");
        m.insert("erl", "Erlang");
        m.insert("ex", "Elixir");
        m.insert("hs", "Haskell");
        m
    };
}

pub fn get_language_name(extension: &str) -> Option<String> {
    EXTENSIONS.get(extension).map(|&s| s.to_string())
}
