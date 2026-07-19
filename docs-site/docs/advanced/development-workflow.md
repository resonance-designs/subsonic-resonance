---
id: development-workflow
title: Development Workflow
sidebar_position: 5
---

This guide covers the enhanced development workflow, code quality tooling, and best practices implemented in the latest version of the Docusaurus Template.

## Development Environment Setup

### Prerequisites

- **Node.js**: Version 18.0 or higher
- **pnpm**: Package manager (recommended over npm/yarn)
- **Git**: For version control
- **VS Code**: Recommended IDE with ESLint and Prettier extensions

### Quick Setup

```bash
# Clone and install dependencies
git clone <repository-url>
cd docusaurus-template
pnpm install

# Run development build
pnpm run start
```

## Code Quality Tools

The template includes comprehensive code quality tooling to ensure consistent, maintainable code:

### ESLint Configuration

Modern ESLint v9 with flat config format (`eslint.config.js`):

```javascript
import js from '@eslint/js';
import tseslint from '@typescript-eslint/eslint-plugin';
import tsParser from '@typescript-eslint/parser';
import globals from 'globals';

export default [
  js.configs.recommended,
  {
    files: ['src/**/*.{ts,tsx}'],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        project: './tsconfig.json',
        sourceType: 'module'
      },
      globals: {
        ...globals.browser,
        ...globals.node
      }
    },
    plugins: {
      '@typescript-eslint': tseslint
    },
    rules: {
      ...tseslint.configs.recommended.rules,
      // Relaxed rules for Docusaurus compatibility
      '@typescript-eslint/no-unused-vars': 'warn',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/ban-ts-comment': 'off'
    }
  },
  {
    ignores: [
      'build/',
      'dist/',
      'node_modules/',
      '.docusaurus/',
      'docusaurus.config.ts',
      'sidebars.ts',
      'scripts/',
      '**/*.d.ts'
    ]
  }
];
```

### Prettier Configuration

Automated code formatting with these settings:

```json
{
  "semi": true,
  "trailingComma": "none",
  "singleQuote": true,
  "printWidth": 80,
  "tabWidth": 2,
  "useTabs": false
}
```

## Available Scripts

The template provides comprehensive scripts for development and quality assurance:

### Development Scripts

```bash
# Start development server with pre-build step
pnpm run start

# Build for production with artifacts output
pnpm run build:prod

# Serve built site locally
pnpm run serve

# Clear Docusaurus cache
pnpm run clear
```

### Quality Assurance Scripts

```bash
# Run ESLint on all TypeScript files
pnpm run lint

# Fix ESLint errors automatically
pnpm run lint:fix

# Format code with Prettier
pnpm run format

# Check formatting without making changes
pnpm run format:check

# Run all quality checks (format + lint + typecheck)
pnpm run check-all

# TypeScript type checking
pnpm run typecheck
```

## Pre-Build Process

The template includes an automated pre-build process that runs before `start` and `build:prod`:

### What It Does

1. **Content Preparation**: Copies root markdown files to `src/pages/`
2. **Navbar Generation**: Auto-generates `src/navbarLinks.ts` from available content
3. **Theme Configuration**: Scans and registers themes from `static/themes/`
4. **Version Management**: Implements date-based versioning (YYYY.MM.DD)

### Pre-Build Script

```typescript
// scripts/pre-build.ts
export class PreBuild {
  private themes: Theme[] = [];

  public generateThemeConfig(): void {
    // Scan static/themes/ directory
    // Extract theme metadata from CSS comments
    // Generate src/themes.ts configuration
  }

  private copyMarkdown(): void {
    // Copy *.md files from root to src/pages/
    // Transform README.md → index.md
  }

  private generateNavbar(): void {
    // Generate src/navbarLinks.ts from available pages
    // Exclude index.md from navigation
  }

  public process(): void {
    this.generateThemeConfig();
    this.copyMarkdown();
    this.generateNavbar();
  }
}
```

## Build Configuration

### Production Build

The production build now outputs to the `./artifacts` directory:

```json
{
  "scripts": {
    "prebuild:prod": "tsx ./scripts/pre-build.ts",
    "build:prod": "docusaurus build --out-dir ./artifacts"
  }
}
```

### Build Process Flow

1. **Pre-build** (`tsx ./scripts/pre-build.ts`)
   - Content preparation
   - Theme scanning
   - Navbar generation
2. **Docusaurus Build** (`docusaurus build --out-dir ./artifacts`)
   - Static site generation
   - Asset optimization
   - Output to artifacts directory

## TypeScript Configuration

Enhanced TypeScript configuration for better Docusaurus compatibility:

```typescript
// tsconfig.json highlights
{
  "extends": "@docusaurus/tsconfig",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "@site/*": ["./src/*"]
    }
  },
  "include": [
    "src/**/*",
    "docs/**/*",
    "scripts/**/*"
  ]
}
```

## Component Architecture

### Static Configuration Classes

The template uses static TypeScript classes for configuration:

```typescript
// Badge Configuration
export class BadgeConfig {
  static templateVariables: TemplateVariables = {
    demoUrl: 'https://docs-template.subzerodev.com',
    user: 'the-running-dev',
    repository: 'Docusaurus-Template'
  };

  static badgeCategories: BadgeCategory[] = [
    // Badge definitions...
  ];
}

// GitHub Links Configuration
export class GitHubLinksConfig {
  static enabled: boolean = true;
  static links: GitHubLink[] = [
    // Link definitions...
  ];
}
```

### Benefits of Static Classes

- **Type Safety**: Full TypeScript IntelliSense and validation
- **No JSON Parsing**: Direct access without file system calls
- **Runtime Modification**: Can be modified at runtime if needed
- **Better IDE Support**: Automatic imports and refactoring support

## Quality Gates

The template enforces quality standards through automated checks:

### Pre-commit Hooks (Husky)

The project uses Husky to automatically run quality checks before commits:

- **Auto-formatting**: Prettier automatically formats code before commit
- **Linting**: ESLint checks are run to catch issues
- **Commit message validation**: Basic validation of commit message format

These hooks are configured in `.husky/pre-commit` and run automatically when you commit.

### Manual Quality Checks

You can also run these checks manually or in your CI/CD pipeline:

```bash
# Format check (fails if code is not formatted)
pnpm run format:check

# Linting (fails on ESLint errors)
pnpm run lint

# Type checking (fails on TypeScript errors)
pnpm run typecheck

# All checks combined
pnpm run check-all
```

### CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Quality Checks
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: npm install -g pnpm
      - run: pnpm install
      - run: pnpm run check-all
```

## Best Practices

### Code Style

1. **Formatting**: Always run `pnpm run format` before committing
2. **Linting**: Fix ESLint warnings with `pnpm run lint:fix`
3. **Type Safety**: Maintain TypeScript strict mode where possible

### Component Development

1. **Configuration**: Use static classes for component configuration
2. **Interfaces**: Define TypeScript interfaces for all props
3. **CSS Classes**: Follow BEM methodology for CSS class naming
4. **Imports**: Use absolute imports with `@site/` prefix

### Documentation

1. **Code Comments**: Document complex logic and component interfaces
2. **Examples**: Provide usage examples in `src/examples/`
3. **Configuration**: Document all configuration options
4. **Migration**: Document breaking changes and migration paths

## Troubleshooting

### Common Issues

**ESLint Errors**:

- Run `pnpm run lint:fix` to automatically fix formatting issues
- Check `eslint.config.js` for rule customizations

**Build Failures**:

- Ensure pre-build step completes successfully
- Check TypeScript errors with `pnpm run typecheck`
- Verify all imports and dependencies are correct

**Theme Issues**:

- Check theme CSS files in `static/themes/` directory
- Verify theme metadata in CSS comments
- Run pre-build to regenerate theme configuration

### Getting Help

1. Check the [troubleshooting documentation](#troubleshooting) section above
2. Review example configurations in `src/examples/`
3. Consult the component-specific documentation
4. Check GitHub issues for similar problems
