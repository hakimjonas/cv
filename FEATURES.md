# CV Generator - Missing Features Roadmap

> A curated set of features to make this CV generator complete for discerning developers

## 🎯 **Current Status**

### ✅ **Implemented Features**
- [x] **Configurable Menu System** - JSON-driven navigation with external link support
- [x] **GitHub Integration** - Auto-fetch projects from user/org repositories
- [x] **Dual Output Generation** - HTML + PDF (Typst) outputs
- [x] **Functional Architecture** - Clean, maintainable Rust code with immutable data structures
- [x] **Responsive Design** - Mobile-friendly interface with theme switching
- [x] **Accessibility** - Screen reader support, keyboard navigation, ARIA attributes

### 🚧 **In Progress**
- [ ] **Typst Theme Configuration** - Currently using hardcoded `grotesk-cv` theme

---

## 🎯 **Missing Features (Priority Order)**

### **1. Static Pages as Markdown** 
**Priority: High** | **Complexity: Medium**

Enable static page generation from markdown files with front matter.

```markdown
---
title: "About Me"
layout: "page"
menu_label: "About"
---

# About Me

I'm a data engineer passionate about functional programming...
```

**Implementation:**
- Add `pulldown-cmark` and `gray_matter` dependencies
- Create `content/pages/` directory structure
- Extend template system to support markdown content
- Add page generation to build pipeline

**Files to modify:**
- `Cargo.toml` - Add markdown dependencies
- `src/main.rs` - Add page generation step
- `src/site_config.rs` - Add page configuration
- `templates/page.html` - Create page template

**Quality Standards Applied:**
- **FP**: Use `im::Vector<Page>` for immutable page collections
- **Testing**: Property-based tests for markdown parsing edge cases
- **Error Handling**: `Result<Page>` with detailed parsing error messages
- **Documentation**: Complete doc comments for all markdown processing functions
- **Accessibility**: Ensure generated HTML maintains semantic structure

---

### **2. Blog Posts as Markdown**
**Priority: High** | **Complexity: Medium**

Replace JavaScript-based blog loading with static markdown generation.

```markdown
---
title: "Building a Functional Language in Rust"
date: "2024-01-15"
tags: ["rust", "compilers", "functional-programming"]
excerpt: "After years of working with Scala..."
---

# Building a Functional Language in Rust

After years of working with Scala and functional programming...
```

**Implementation:**
- Create `content/blog/` directory structure
- Add blog post parsing and generation
- Update blog template to use static content
- Add tag-based filtering and pagination

**Files to modify:**
- `src/blog_generator.rs` - New module for blog processing
- `templates/blog.html` - Update to use static content
- `templates/blog_post.html` - Individual post template

**Quality Standards Applied:**
- **FP**: Immutable `BlogPost` structs with `im::Vector<Tag>`
- **Testing**: Property-based tests for front matter parsing
- **Error Handling**: `Result<BlogPost>` with validation errors
- **Documentation**: Complete API docs for blog processing functions
- **Accessibility**: Proper heading hierarchy and semantic HTML

---

### **3. Typst Theme Configuration** ✅ **COMPLETED**
**Priority: High** | **Complexity: Low**

Make Typst PDF theme configurable instead of hardcoded.

```json
{
  "typst": {
    "theme": {
      "name": "grotesk-cv",
      "version": "1.0.2", 
      "source": "@preview/grotesk-cv"
    },
    "customization": {
      "colors": {
        "fill": "#f4f1eb",
        "accent": "#d4d2cc",
        "text_light": "#ededef", 
        "text_medium": "#78787e",
        "text_dark": "#3c3c42"
      },
      "layout": {
        "paper_size": "a4",
        "left_pane_width": "71%",
        "font": "HK Grotesk",
        "font_size": "9pt"
      }
    }
  }
}
```

**Implementation:**
- Add `TypstConfig` to `site_config.rs`
- Modify `typst_generator/markup.rs` to use config
- Add popular theme presets (grotesk-cv, modern-cv, academic-cv, minimal-cv)
- Update `generate_imports()` to be dynamic

**Files to modify:**
- `src/site_config.rs` - Add TypstConfig struct
- `src/typst_generator/markup.rs` - Make theme configurable
- `config/site.json` - Add typst configuration section

**Quality Standards Applied:**
- **FP**: Immutable `TypstConfig` with validation
- **Testing**: Unit tests for theme validation and fallbacks
- **Error Handling**: `Result<TypstConfig>` with theme validation errors
- **Documentation**: Complete docs for theme configuration options
- **Performance**: Efficient theme switching without recompilation

---

### **4. Nerd Font Selection**
**Priority: Medium** | **Complexity: Low**

Allow users to choose their preferred Nerd Font for the website.

```json
{
  "fonts": {
    "primary": "JetBrainsMono",
    "fallback": "FiraCode",
    "source": "nerd-fonts"
  }
}
```

**Implementation:**
- Add font configuration to site config
- Update CSS generation to use selected font
- Add font loading from CDN or local files
- Provide popular font presets

**Files to modify:**
- `src/site_config.rs` - Add FontConfig struct
- `static/css/base/variables.css` - Make font configurable
- `config/site.json` - Add font configuration

---

### **5. Terminal Colorscheme Integration**
**Priority: Medium** | **Complexity: Medium**

Integrate popular terminal colorschemes for website theming.

```json
{
  "colorscheme": {
    "name": "tokyonight",
    "source": "ghostty-colors",
    "url": "https://github.com/ghostty-org/ghostty-colors"
  }
}
```

**Implementation:**
- Add colorscheme configuration
- Create colorscheme-to-CSS mapping
- Support popular schemes (tokyonight, catppuccin, dracula, etc.)
- Add colorscheme preview functionality

**Files to modify:**
- `src/site_config.rs` - Add ColorschemeConfig struct
- `src/css_generator.rs` - New module for dynamic CSS generation
- `config/site.json` - Add colorscheme configuration

---

## 🎯 **Feature Completion Criteria**

### **Core Functionality**
- [ ] All markdown content renders correctly
- [ ] Static pages integrate with navigation
- [ ] Blog posts support tags and filtering
- [ ] Typst themes are fully configurable
- [ ] Font selection works across all pages
- [ ] Colorschemes apply consistently

### **Developer Experience**
- [ ] Configuration is intuitive and well-documented
- [ ] Error messages are helpful and actionable
- [ ] Build process is fast and reliable
- [ ] Generated output is clean and professional

### **Quality Assurance**
- [ ] All features have comprehensive tests
- [ ] Documentation is complete and up-to-date
- [ ] Examples demonstrate all capabilities
- [ ] Performance is optimized

---

## 🚀 **Implementation Strategy**

### **Phase 1: Content Management** (Features 1-2)
- Static pages and blog posts as markdown
- Foundation for all content-driven features

### **Phase 2: PDF Customization** (Feature 3)
- Typst theme configuration
- Complete the PDF generation story

### **Phase 3: Visual Customization** (Features 4-5)
- Font and colorscheme integration
- Polish the developer aesthetic

### **Phase 4: Polish & Documentation**
- Comprehensive testing
- Documentation and examples
- Performance optimization

---

## 📝 **Development Standards**

### **Functional Programming Principles**
- **Immutable data structures** - Use `im::Vector` and `im::HashMap`
- **Pure functions** - No side effects, predictable outputs
- **Method chaining** - Use `.pipe()` for composable transformations
- **Error handling** - Comprehensive `Result<T>` usage with `anyhow`
- **Type safety** - Leverage Rust's type system for correctness

### **Testing Requirements**
- **Unit tests** - Every public function must have tests
- **Property-based testing** - Use `proptest` for complex data structures
- **Integration tests** - End-to-end functionality verification
- **Test coverage** - Maintain high coverage for all new code
- **Edge case handling** - Test error conditions and boundary cases

### **Code Quality Standards**
- **Documentation** - Comprehensive doc comments for all public APIs
- **Error messages** - Helpful, actionable error messages with context
- **Performance** - Efficient algorithms, minimal allocations
- **Accessibility** - ARIA attributes, keyboard navigation, screen reader support
- **Security** - Input validation, safe string handling

### **Architecture Principles**
- **Configuration-driven** - JSON configs, not hardcoded values
- **Template-based generation** - Askama templates for all output
- **Separation of concerns** - Clear module boundaries
- **Extensibility** - Easy to add new features without breaking existing ones
- **Maintainability** - Clean, readable code that future developers can understand

### **Quality Assurance Checklist**
For every new feature:
- [ ] **FP Principles** - Immutable data, pure functions, method chaining
- [ ] **Comprehensive Tests** - Unit, integration, and property-based tests
- [ ] **Documentation** - Complete doc comments and examples
- [ ] **Error Handling** - Proper `Result<T>` usage with helpful messages
- [ ] **Accessibility** - ARIA attributes and keyboard navigation
- [ ] **Performance** - Efficient implementation with minimal allocations
- [ ] **Configuration** - JSON-driven, not hardcoded
- [ ] **Type Safety** - Leverage Rust's type system
- [ ] **Edge Cases** - Handle error conditions gracefully
- [ ] **Integration** - Works seamlessly with existing features

---

**Last Updated:** 2024-09-25  
**Status:** Planning Phase - Ready for Implementation