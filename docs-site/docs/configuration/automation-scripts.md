---
id: automation-scripts
title: Automation
sidebar_position: 3
---

The template includes two PowerShell scripts to streamline the development workflow:

### `template-setup.ps1` - Template Configuration

Sets up badge configuration and handles any remaining template files.

**Usage:**

```powershell
# Run from template directory
.\template-setup.ps1

# Or specify a different project directory
.\template-setup.ps1 -projectDir "C:\path\to\project"
```

**What it does:**

- Sets up static configuration files for badges and Giscus comments
- Processes any remaining `.example` files in the project
- Cleans up template files after successful setup
- Provides colored console output for progress tracking

**Perfect for:**

- Setting up static configuration for your project
- Final template cleanup after customization
- Ensuring all configuration files are properly set up

### `template-build.ps1` - Development Server Launcher

**⚠️ Note:** This script has been simplified and now runs the development server directly in the current terminal rather than a separate window.

Automates the development workflow with comprehensive PowerShell documentation.

**Usage:**

```powershell
# Run from template directory (uses current directory)
.\template-build.ps1

# Or specify a different app directory
.\template-build.ps1 -appDir ".\my-docs-site"
```

**What it does:**

- Resolves full path to the documentation directory
- Installs dependencies using `pnpm install`
- Runs pre-build steps (`pnpm run prebuild` - content preparation and versioning)
- Starts Docusaurus development server (`pnpm start`)
- Includes comprehensive PowerShell help documentation

**Features:**

- 📖 **Full PowerShell Help** - Run `Get-Help .\template-build.ps1 -Full` for complete documentation
- 🔧 **Parameter Validation** - Validates directory paths and provides helpful errors
- 🚀 **pnpm Integration** - Uses pnpm for faster dependency management
- ⚙️ **Pre-build Integration** - Automatically runs version generation
- 🎨 **Visual Feedback** - Colored progress indicators and status messages

**Requirements:**

- `pnpm` package manager installed and available in PATH
- PowerShell execution policy allowing script execution
- Valid `package.json` with required scripts (`prebuild`, `start`)
- PowerShell 5.0 or higher
