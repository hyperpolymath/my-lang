//! Site generator for My SSG
//!
//! Orchestrates the build process: reading content, applying templates,
//! and generating the static site.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::config::Config;
use crate::markdown;
use crate::template::{self, Context, ContextValue};

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("markdown error: {0}")]
    Markdown(#[from] markdown::MarkdownError),
    #[error("template error: {0}")]
    Template(#[from] template::TemplateError),
    #[error("configuration error: {0}")]
    Config(String),
}

/// Build statistics
#[derive(Debug, Default)]
pub struct BuildStats {
    pub pages: usize,
    pub posts: usize,
    pub static_files: usize,
    pub output_dir: String,
}

/// Content page
#[derive(Debug)]
struct Page {
    pub path: PathBuf,
    pub url: String,
    pub title: String,
    pub date: Option<String>,
    pub template: String,
    pub content: String,
    pub html: String,
    pub tags: Vec<String>,
    pub summary: Option<String>,
    pub is_post: bool,
}

/// Site generator
pub struct Generator {
    config: Config,
    templates: HashMap<String, String>,
}

impl Generator {
    pub fn new(config: Config) -> Self {
        Generator {
            config,
            templates: HashMap::new(),
        }
    }

    /// Build the complete static site
    pub fn build(&self) -> Result<BuildStats, GeneratorError> {
        let mut stats = BuildStats::default();
        stats.output_dir = self.config.build.output_dir.display().to_string();

        // Create output directory
        fs::create_dir_all(&self.config.build.output_dir)?;

        // Load templates
        let templates = self.load_templates()?;

        // Collect all content
        let mut pages = self.collect_content()?;

        // Sort posts by date (newest first)
        pages.sort_by(|a, b| {
            let date_a = a.date.as_deref().unwrap_or("");
            let date_b = b.date.as_deref().unwrap_or("");
            date_b.cmp(date_a)
        });

        // Separate posts from pages
        let posts: Vec<&Page> = pages.iter().filter(|p| p.is_post).collect();
        let regular_pages: Vec<&Page> = pages.iter().filter(|p| !p.is_post).collect();

        // Create base context with site info
        let site_context = self.create_site_context(&posts);

        // Generate pages
        for page in &regular_pages {
            self.generate_page(page, &templates, &site_context)?;
            stats.pages += 1;
        }

        // Generate posts
        for post in &posts {
            self.generate_page(post, &templates, &site_context)?;
            stats.posts += 1;
        }

        // Generate posts index
        self.generate_posts_index(&posts, &templates, &site_context)?;

        // Copy static files
        stats.static_files = self.copy_static_files()?;

        Ok(stats)
    }

    /// Load all templates from the templates directory
    fn load_templates(&self) -> Result<HashMap<String, String>, GeneratorError> {
        let mut templates = HashMap::new();
        let templates_dir = &self.config.build.templates_dir;

        if !templates_dir.exists() {
            // No templates dir, use defaults
            templates.insert("base".to_string(), default_base_template());
            templates.insert("post".to_string(), default_post_template());
            templates.insert("index".to_string(), default_index_template());
            return Ok(templates);
        }

        for entry in fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "html").unwrap_or(false) {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                let content = fs::read_to_string(&path)?;
                templates.insert(name.to_string(), content);
            }
        }

        // Ensure we have base templates
        if !templates.contains_key("base") {
            templates.insert("base".to_string(), default_base_template());
        }

        Ok(templates)
    }

    /// Collect all content files
    fn collect_content(&self) -> Result<Vec<Page>, GeneratorError> {
        let mut pages = Vec::new();
        let content_dir = &self.config.build.content_dir;

        if !content_dir.exists() {
            return Ok(pages);
        }

        self.collect_content_recursive(content_dir, &mut pages)?;
        Ok(pages)
    }

    fn collect_content_recursive(
        &self,
        dir: &Path,
        pages: &mut Vec<Page>,
    ) -> Result<(), GeneratorError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.collect_content_recursive(&path, pages)?;
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                let page = self.parse_content_file(&path)?;
                // Skip drafts
                if !page.template.is_empty() || !page.content.is_empty() {
                    pages.push(page);
                }
            }
        }
        Ok(())
    }

    /// Parse a content file into a Page
    fn parse_content_file(&self, path: &Path) -> Result<Page, GeneratorError> {
        let content = fs::read_to_string(path)?;
        let doc = markdown::parse(&content)?;

        // Determine if this is a post (in posts directory)
        let is_post = path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n == "posts")
            .unwrap_or(false);

        // Generate URL from path
        let relative_path = path
            .strip_prefix(&self.config.build.content_dir)
            .unwrap_or(path);
        let url = path_to_url(relative_path);

        // Get title from frontmatter or filename
        let title = doc.frontmatter.title.clone().unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });

        // Get template from frontmatter or default
        let template = doc.frontmatter.template.clone().unwrap_or_else(|| {
            if is_post {
                "post".to_string()
            } else {
                "base".to_string()
            }
        });

        Ok(Page {
            path: path.to_path_buf(),
            url,
            title,
            date: doc.frontmatter.date,
            template,
            content: doc.content,
            html: doc.html,
            tags: doc.frontmatter.tags,
            summary: doc.frontmatter.summary,
            is_post,
        })
    }

    /// Create the base site context
    fn create_site_context(&self, posts: &[&Page]) -> Context {
        let mut ctx = Context::new();

        // Site info
        let mut site = HashMap::new();
        site.insert(
            "title".to_string(),
            ContextValue::String(self.config.site.title.clone()),
        );
        site.insert(
            "description".to_string(),
            ContextValue::String(self.config.site.description.clone()),
        );
        site.insert(
            "base_url".to_string(),
            ContextValue::String(self.config.site.base_url.clone()),
        );
        site.insert(
            "language".to_string(),
            ContextValue::String(self.config.site.language.clone()),
        );
        ctx.insert("site", ContextValue::Object(site));

        // Current year
        ctx.insert("year", ContextValue::String("2025".to_string()));

        // All posts
        let posts_array: Vec<ContextValue> = posts
            .iter()
            .map(|p| {
                let mut post_obj = HashMap::new();
                post_obj.insert("title".to_string(), ContextValue::String(p.title.clone()));
                post_obj.insert("url".to_string(), ContextValue::String(p.url.clone()));
                post_obj.insert(
                    "date".to_string(),
                    ContextValue::String(p.date.clone().unwrap_or_default()),
                );
                post_obj.insert(
                    "date_formatted".to_string(),
                    ContextValue::String(format_date(p.date.as_deref())),
                );
                if let Some(ref summary) = p.summary {
                    post_obj.insert("summary".to_string(), ContextValue::String(summary.clone()));
                }
                post_obj.insert(
                    "tags".to_string(),
                    ContextValue::Array(
                        p.tags.iter().map(|t| ContextValue::String(t.clone())).collect(),
                    ),
                );
                ContextValue::Object(post_obj)
            })
            .collect();
        ctx.insert("posts", ContextValue::Array(posts_array));

        ctx
    }

    /// Generate a single page
    fn generate_page(
        &self,
        page: &Page,
        templates: &HashMap<String, String>,
        site_context: &Context,
    ) -> Result<(), GeneratorError> {
        // Build page context
        let mut ctx = site_context.clone();

        // Page info
        let mut page_obj = HashMap::new();
        page_obj.insert("title".to_string(), ContextValue::String(page.title.clone()));
        page_obj.insert("url".to_string(), ContextValue::String(page.url.clone()));
        page_obj.insert(
            "date".to_string(),
            ContextValue::String(page.date.clone().unwrap_or_default()),
        );
        page_obj.insert(
            "date_formatted".to_string(),
            ContextValue::String(format_date(page.date.as_deref())),
        );
        page_obj.insert(
            "tags".to_string(),
            ContextValue::Array(
                page.tags.iter().map(|t| ContextValue::String(t.clone())).collect(),
            ),
        );
        ctx.insert("page", ContextValue::Object(page_obj));

        // Content
        ctx.insert("content", ContextValue::String(page.html.clone()));

        // Get template
        let template_content = templates
            .get(&page.template)
            .or_else(|| templates.get("base"))
            .ok_or_else(|| GeneratorError::Config("no template found".to_string()))?;

        // Handle template inheritance
        let final_template = if template_content.contains("{% extends") {
            // Simple inheritance: replace {{ content }} in base with page template content
            let base = templates.get("base").cloned().unwrap_or_default();
            base.replace("{{ content }}", template_content)
        } else {
            template_content.clone()
        };

        // Render template
        let html = template::render(&final_template, &ctx)?;

        // Write output file
        let output_path = self.url_to_output_path(&page.url);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, html)?;

        Ok(())
    }

    /// Generate the posts index page
    fn generate_posts_index(
        &self,
        posts: &[&Page],
        templates: &HashMap<String, String>,
        site_context: &Context,
    ) -> Result<(), GeneratorError> {
        if posts.is_empty() {
            return Ok(());
        }

        let mut ctx = site_context.clone();

        let mut page_obj = HashMap::new();
        page_obj.insert("title".to_string(), ContextValue::String("Blog".to_string()));
        page_obj.insert("url".to_string(), ContextValue::String("/posts/".to_string()));
        ctx.insert("page", ContextValue::Object(page_obj));

        // Build posts list HTML
        let mut posts_html = String::from("<h1>Blog Posts</h1>\n<div class=\"posts-list\">\n");
        for post in posts {
            posts_html.push_str(&format!(
                "<article class=\"post-preview\">\n\
                 <h2><a href=\"{}\">{}</a></h2>\n\
                 <time datetime=\"{}\">{}</time>\n\
                 </article>\n",
                post.url,
                post.title,
                post.date.as_deref().unwrap_or(""),
                format_date(post.date.as_deref())
            ));
        }
        posts_html.push_str("</div>");

        ctx.insert("content", ContextValue::String(posts_html));

        let template_content = templates.get("base").cloned().unwrap_or_default();
        let html = template::render(&template_content, &ctx)?;

        let output_path = self.config.build.output_dir.join("posts/index.html");
        fs::create_dir_all(output_path.parent().unwrap())?;
        fs::write(&output_path, html)?;

        Ok(())
    }

    /// Copy static files to output
    fn copy_static_files(&self) -> Result<usize, GeneratorError> {
        let static_dir = &self.config.build.static_dir;
        if !static_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        self.copy_dir_recursive(static_dir, &self.config.build.output_dir, &mut count)?;
        Ok(count)
    }

    fn copy_dir_recursive(
        &self,
        src: &Path,
        dest: &Path,
        count: &mut usize,
    ) -> Result<(), GeneratorError> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let relative = path.strip_prefix(src).unwrap_or(&path);
            let dest_path = dest.join(relative);

            if path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                self.copy_dir_recursive(&path, &dest_path, count)?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&path, &dest_path)?;
                *count += 1;
            }
        }
        Ok(())
    }

    /// Convert URL to output file path
    fn url_to_output_path(&self, url: &str) -> PathBuf {
        let clean_url = url.trim_start_matches('/');

        if clean_url.is_empty() || clean_url == "/" {
            self.config.build.output_dir.join("index.html")
        } else if clean_url.ends_with('/') {
            self.config.build.output_dir.join(clean_url).join("index.html")
        } else {
            self.config.build.output_dir.join(format!("{}/index.html", clean_url))
        }
    }
}

/// Convert a file path to a URL
fn path_to_url(path: &Path) -> String {
    let mut url = String::from("/");

    let path_str = path.to_string_lossy();
    let without_ext = path_str
        .strip_suffix(".md")
        .unwrap_or(&path_str);

    // Handle index files
    if without_ext == "index" || without_ext.ends_with("/index") {
        url.push_str(&without_ext.replace("/index", ""));
    } else {
        url.push_str(without_ext);
    }

    // Ensure trailing slash for directories
    if !url.ends_with('/') && !url.contains('.') {
        url.push('/');
    }

    // Clean up double slashes
    while url.contains("//") {
        url = url.replace("//", "/");
    }

    url
}

/// Format a date string for display
fn format_date(date: Option<&str>) -> String {
    date.map(|d| {
        // Simple ISO date formatting
        // Full implementation would parse and format properly
        d.to_string()
    })
    .unwrap_or_default()
}

/// Default base template
fn default_base_template() -> String {
    r#"<!DOCTYPE html>
<html lang="{{ site.language }}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ page.title }} - {{ site.title }}</title>
    <style>
        body { font-family: system-ui, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; }
        header { border-bottom: 1px solid #eee; padding-bottom: 1rem; margin-bottom: 2rem; }
        nav a { margin-right: 1rem; }
        footer { margin-top: 2rem; padding-top: 1rem; border-top: 1px solid #eee; color: #666; }
    </style>
</head>
<body>
    <header>
        <nav>
            <a href="/">{{ site.title }}</a>
            <a href="/posts/">Blog</a>
        </nav>
    </header>
    <main>{{ content }}</main>
    <footer>&copy; {{ year }} {{ site.title }}. Built with My SSG.</footer>
</body>
</html>"#
        .to_string()
}

/// Default post template
fn default_post_template() -> String {
    r#"<article>
    <header>
        <h1>{{ page.title }}</h1>
        <time>{{ page.date_formatted }}</time>
    </header>
    {{ content }}
</article>"#
        .to_string()
}

/// Default index template
fn default_index_template() -> String {
    r#"<div class="home">
    <h1>{{ site.title }}</h1>
    <p>{{ site.description }}</p>
    {{ content }}
</div>"#
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_to_url() {
        assert_eq!(path_to_url(Path::new("index.md")), "/");
        assert_eq!(path_to_url(Path::new("about.md")), "/about/");
        assert_eq!(path_to_url(Path::new("posts/hello.md")), "/posts/hello/");
    }
}
