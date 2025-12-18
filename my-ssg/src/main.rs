//! My SSG - Static Site Generator powered by My Language
//!
//! A fast, AI-native static site generator that uses My Language for templating.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

mod config;
mod generator;
mod markdown;
mod template;

use config::Config;
use generator::Generator;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "build" => {
            let config_path = args.get(2).map(String::as_str).unwrap_or("ssg.toml");
            build_site(config_path);
        }
        "new" => {
            if args.len() < 3 {
                eprintln!("Error: 'new' command requires a project name");
                process::exit(1);
            }
            create_new_project(&args[2]);
        }
        "serve" => {
            let config_path = args.get(2).map(String::as_str).unwrap_or("ssg.toml");
            serve_site(config_path);
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        "version" | "--version" | "-v" => {
            println!("My SSG v0.1.0");
            println!("Powered by My Language v0.1.0");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("My SSG - Static Site Generator powered by My Language");
    eprintln!();
    eprintln!("Usage: my-ssg <command> [options]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  build [config]    Build the static site (default: ssg.toml)");
    eprintln!("  new <name>        Create a new SSG project");
    eprintln!("  serve [config]    Build and serve locally");
    eprintln!("  help              Show this help message");
    eprintln!("  version           Show version information");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  my-ssg new my-blog");
    eprintln!("  my-ssg build");
    eprintln!("  my-ssg serve");
}

fn build_site(config_path: &str) {
    println!("Building site...");

    let config = match Config::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            // If no config file, use defaults
            eprintln!("Note: No config file found, using defaults: {}", e);
            Config::default()
        }
    };

    let generator = Generator::new(config);

    match generator.build() {
        Ok(stats) => {
            println!();
            println!("Build complete!");
            println!("  Pages generated: {}", stats.pages);
            println!("  Posts generated: {}", stats.posts);
            println!("  Static files copied: {}", stats.static_files);
            println!("  Output directory: {}", stats.output_dir);
        }
        Err(e) => {
            eprintln!("Build failed: {}", e);
            process::exit(1);
        }
    }
}

fn create_new_project(name: &str) {
    println!("Creating new SSG project: {}", name);

    let base_path = PathBuf::from(name);

    // Create directory structure
    let dirs = [
        "",
        "content",
        "content/posts",
        "templates",
        "static",
        "static/css",
        "static/js",
    ];

    for dir in &dirs {
        let path = base_path.join(dir);
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("Failed to create directory {:?}: {}", path, e);
            process::exit(1);
        }
    }

    // Create config file
    let config_content = format!(
        r#"# My SSG Configuration
# Generated for: {}

[site]
title = "{}"
description = "A site built with My SSG"
base_url = "https://example.com"
language = "en"

[build]
content_dir = "content"
templates_dir = "templates"
static_dir = "static"
output_dir = "_site"

[features]
syntax_highlighting = true
ai_summaries = false
"#,
        name, name
    );
    fs::write(base_path.join("ssg.toml"), config_content).expect("Failed to write config");

    // Create base template
    let base_template = r#"<!DOCTYPE html>
<html lang="{{ site.language }}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ page.title }} - {{ site.title }}</title>
    <link rel="stylesheet" href="/css/style.css">
</head>
<body>
    <header>
        <nav>
            <a href="/">{{ site.title }}</a>
            <ul>
                <li><a href="/">Home</a></li>
                <li><a href="/posts/">Blog</a></li>
                <li><a href="/about/">About</a></li>
            </ul>
        </nav>
    </header>

    <main>
        {{ content }}
    </main>

    <footer>
        <p>&copy; {{ year }} {{ site.title }}. Built with My SSG.</p>
    </footer>
</body>
</html>
"#;
    fs::write(base_path.join("templates/base.html"), base_template)
        .expect("Failed to write template");

    // Create post template
    let post_template = r#"{% extends "base.html" %}

<article class="post">
    <header>
        <h1>{{ page.title }}</h1>
        <time datetime="{{ page.date }}">{{ page.date_formatted }}</time>
        {% if page.tags %}
        <div class="tags">
            {% for tag in page.tags %}
            <span class="tag">{{ tag }}</span>
            {% endfor %}
        </div>
        {% endif %}
    </header>

    <div class="content">
        {{ content }}
    </div>
</article>
"#;
    fs::write(base_path.join("templates/post.html"), post_template)
        .expect("Failed to write template");

    // Create index template
    let index_template = r#"{% extends "base.html" %}

<div class="home">
    <h1>Welcome to {{ site.title }}</h1>
    <p>{{ site.description }}</p>

    <section class="recent-posts">
        <h2>Recent Posts</h2>
        {% for post in posts limit:5 %}
        <article class="post-preview">
            <h3><a href="{{ post.url }}">{{ post.title }}</a></h3>
            <time datetime="{{ post.date }}">{{ post.date_formatted }}</time>
            {% if post.summary %}
            <p>{{ post.summary }}</p>
            {% endif %}
        </article>
        {% endfor %}
    </section>
</div>
"#;
    fs::write(base_path.join("templates/index.html"), index_template)
        .expect("Failed to write template");

    // Create default CSS
    let css_content = r#"/* My SSG Default Styles */

:root {
    --primary-color: #2563eb;
    --text-color: #1f2937;
    --bg-color: #ffffff;
    --secondary-bg: #f3f4f6;
    --border-color: #e5e7eb;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    line-height: 1.6;
    color: var(--text-color);
    background-color: var(--bg-color);
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
}

header {
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border-color);
}

header nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
}

header nav a {
    font-weight: bold;
    font-size: 1.25rem;
    text-decoration: none;
    color: var(--primary-color);
}

header nav ul {
    display: flex;
    list-style: none;
    gap: 1rem;
}

header nav ul a {
    font-weight: normal;
    font-size: 1rem;
}

main {
    min-height: 60vh;
}

h1, h2, h3, h4, h5, h6 {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
    line-height: 1.3;
}

h1 { font-size: 2rem; }
h2 { font-size: 1.5rem; }
h3 { font-size: 1.25rem; }

p {
    margin-bottom: 1rem;
}

a {
    color: var(--primary-color);
}

code {
    background: var(--secondary-bg);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-size: 0.9em;
}

pre {
    background: var(--secondary-bg);
    padding: 1rem;
    border-radius: 5px;
    overflow-x: auto;
    margin-bottom: 1rem;
}

pre code {
    background: none;
    padding: 0;
}

blockquote {
    border-left: 3px solid var(--primary-color);
    padding-left: 1rem;
    margin: 1rem 0;
    color: #6b7280;
}

.post-preview {
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border-color);
}

.post-preview h3 {
    margin-top: 0;
}

.post-preview time {
    color: #6b7280;
    font-size: 0.9rem;
}

.tags {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
}

.tag {
    background: var(--secondary-bg);
    padding: 0.2rem 0.5rem;
    border-radius: 3px;
    font-size: 0.8rem;
}

footer {
    margin-top: 3rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border-color);
    color: #6b7280;
    font-size: 0.9rem;
    text-align: center;
}

@media (max-width: 600px) {
    body {
        padding: 1rem;
    }

    header nav {
        flex-direction: column;
        gap: 1rem;
    }
}
"#;
    fs::write(base_path.join("static/css/style.css"), css_content)
        .expect("Failed to write CSS");

    // Create sample index page
    let index_content = r#"---
title: Home
template: index
---

Welcome to your new site powered by My SSG!
"#;
    fs::write(base_path.join("content/index.md"), index_content)
        .expect("Failed to write index");

    // Create sample post
    let post_content = r#"---
title: Hello World
date: 2025-01-01
template: post
tags: [welcome, my-ssg]
---

# Hello World!

This is your first blog post. My SSG makes it easy to create static sites with the power of **My Language**.

## Features

- Markdown support
- Custom templates with My Language expressions
- AI-powered features (when enabled)
- Fast build times

## Code Example

Here's some code in My Language:

```mylang
fn greet(name: String) -> String {
    return "Hello, " + name + "!";
}

fn main() {
    let message = greet("World");
    println(message);
}
```

Happy blogging!
"#;
    fs::write(
        base_path.join("content/posts/hello-world.md"),
        post_content,
    )
    .expect("Failed to write post");

    // Create about page
    let about_content = r#"---
title: About
template: base
---

# About

This site is built with [My SSG](https://github.com/hyperpolymath/my-lang), a static site generator powered by My Language.

## My Language

My Language is a programming language with first-class AI integration, featuring:

- Type safety with AI type constraints
- Prompt templates
- Effect tracking
- Memory safety
"#;
    fs::write(base_path.join("content/about.md"), about_content)
        .expect("Failed to write about page");

    println!();
    println!("Project created successfully!");
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  my-ssg build");
    println!();
}

fn serve_site(config_path: &str) {
    // First build
    build_site(config_path);

    // Simple static file server
    println!();
    println!("Starting development server...");
    println!("Serving at: http://localhost:8080");
    println!("Press Ctrl+C to stop");
    println!();

    // Note: A full implementation would use a proper HTTP server
    // For now, we just build and tell the user to use another server
    println!("Note: Use any static file server to serve the _site directory:");
    println!("  python3 -m http.server 8080 --directory _site");
    println!("  npx serve _site");
}
