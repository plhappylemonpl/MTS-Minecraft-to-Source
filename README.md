# MTS - Minecraft to Source

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-2.0-green.svg)
![Minecraft](https://img.shields.io/badge/Minecraft-1.21.8-brightgreen.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey.svg)

**A modern, high-performance application that converts Minecraft worlds to Source Engine VMF format**

Compatible with **Garry's Mod**, **Counter-Strike: Source**, **Team Fortress 2**, and other Source engine games

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Building](#-building-from-source) • [Textures](#-textures)

</div>

---

## 🎯 Features

### Version 2.0 - Complete Rewrite in Rust

- **⚡ Blazing Fast**: Built with Rust and Tauri for maximum performance
- **🎨 Modern UI**: Sleek, responsive interface with real-time progress tracking
- **🔧 Greedy Mesh Optimization**: Optional optimization reduces brush count by 80-95%
- **📦 Multi-threaded Processing**: Efficient chunk reading and batch processing
- **🎮 Block Properties Support**: Handles block states (e.g., `snowy=true`, `lit=true`)
- **💾 Memory Efficient**: Smart memory management with configurable batch sizes
- **🌍 Flexible Conversion Modes**:
  - **Dynamic Mode**: Auto-detects and converts only non-empty chunks
  - **Static Mode**: Convert specific coordinate ranges
- **📊 Real-time Progress**: Three-level progress tracking (Overall, Stage, Detail)

### Supported Minecraft Version

- ✅ Minecraft **1.21.8** (fully tested)
- ✅ Full-block texture support: **~90%**
- ✅ Block properties and variants

---

## 🚀 Installation

### Download Pre-built Binaries

**Coming Soon**: Pre-compiled executables for Windows and Linux will be available in the [Releases](https://github.com/plhappylemonpl/MTS-Minecraft-to-Source/releases) section.

### System Requirements

- **RAM**: Minimum 4GB, recommended 8GB+ for large worlds
- **Storage**: Depends on world size (approximately 10-20% of original world size)
- **OS**: 
  - Windows 10/11 (64-bit)
  - Linux (Ubuntu 20.04+, or equivalent)

---

## 📦 Textures

### Download Texture Pack

**Download**: [MC 1.21.8 Texture Pack (MEGA)](https://mega.nz/file/CpBgRAJT#i8d3DTKHNDWsEMVzr1dTqCCEGF1r6qFkfrx1yvH2ulc)

### Installation Path

#### For Garry's Mod:
```
Steam/steamapps/common/GarrysMod/garrysmod/materials/mc_1.21.8/
```

#### For Counter-Strike: Source:
```
Steam/steamapps/common/Counter-Strike Source/cstrike/materials/mc_1.21.8/
```

#### For Team Fortress 2:
```
Steam/steamapps/common/Team Fortress 2/tf/materials/mc_1.21.8/
```

> **Note**: Make sure to extract the textures to the `materials` folder, not `maps` or `models`!

---

## 🎮 Usage

### Quick Start

1. **Launch MTS** application
2. **Select your Minecraft world** folder (containing `level.dat`)
3. **Choose output location** for the VMF file
4. **Select conversion mode**:
   - **Dynamic**: Automatically detects and converts non-empty chunks
   - **Static**: Define specific coordinate ranges
5. **Enable/Disable Optimization**: 
   - ✅ **Enabled**: Reduces brush count significantly (recommended for large builds)
   - ❌ **Disabled**: 1:1 block-to-brush conversion (faster, but larger files)
6. **Click "Start Conversion"** and wait for completion

### Conversion Modes Explained

#### Dynamic Mode
- Automatically skips empty chunks
- Best for: Survival worlds, scattered builds
- Pros: No coordinate input needed, efficient
- Cons: May include some unwanted areas

#### Static Mode (Coordinate Range)
- Manually define X/Z coordinate boundaries
- Best for: Specific builds, creative plots
- Pros: Precise control over conversion area
- Cons: Requires knowing coordinates

### Optimization Toggle

| Mode | Brush Count | File Size | Compile Time | Best For |
|------|-------------|-----------|--------------|----------|
| **Optimized** | 5-20% of original | Small | Fast | Large worlds, performance |
| **Standard** | 100% (1:1) | Large | Slow | Small builds, max detail |

**Recommendation**: Always use optimization for worlds larger than 100x100 blocks.

---

## 🔨 Recommended Tools

### Hammer++ (Enhanced Map Editor)

For the best map editing experience, use **Hammer++**, an improved version of the standard Hammer Editor:

**Download**: [https://ficool2.github.io/HammerPlusPlus-Website/](https://ficool2.github.io/HammerPlusPlus-Website/)

### Hammer Editor Settings

For optimal texture scaling when working with MTS-generated maps:

```
Tools > Options > Game Configurations > Default texture scale: 0.1562
```

This ensures textures display at the correct scale (40 units per block).

---

## 🛠️ Building from Source

### Prerequisites

#### Windows

1. **Install Rust** (latest stable):
   ```bash
   # Download from: https://rustup.rs/
   # Or use:
   winget install Rustlang.Rustup
   ```

2. **Install Node.js** (v18 or later):
   ```bash
   # Download from: https://nodejs.org/
   # Or use:
   winget install OpenJS.NodeJS
   ```

3. **Install Bun** (JavaScript runtime):
   ```bash
   powershell -c "irm bun.sh/install.ps1 | iex"
   ```

4. **Visual Studio Build Tools** (C++ compiler):
   - Download: [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
   - Required components:
     - ✅ Desktop development with C++
     - ✅ Windows 10/11 SDK

#### Linux (Ubuntu/Debian)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js and npm
sudo apt update
sudo apt install nodejs npm

# Install Bun
curl -fsSL https://bun.sh/install | bash

# Install dependencies
sudo apt install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### Build Steps

1. **Clone the repository**:
   ```bash
   git clone https://github.com/plhappylemonpl/MTS-Minecraft-to-Source.git
   cd MTS-Minecraft-to-Source
   ```

2. **Install dependencies**:
   ```bash
   bun install
   ```

3. **Build the application**:
   
   **Development mode** (with hot-reload):
   ```bash
   bun run tauri dev
   ```
   
   **Production build**:
   ```bash
   bun run tauri build
   ```

4. **Find compiled binaries**:
   - **Windows**: `src-tauri/target/release/mts.exe`
   - **Linux**: `src-tauri/target/release/mts`

---

## 🧪 Technical Details

### Architecture

- **Frontend**: React with TypeScript for reactive UI
- **Backend**: Rust with Tauri for native performance
- **World Reading**: `fastanvil` crate for efficient Anvil format parsing
- **Optimization**: Custom greedy mesh algorithm for brush reduction

### Performance Metrics

Tested on a mid-range PC (Intel i5, 16GB RAM):

| World Size | Blocks | Optimization | Brushes | Time | File Size |
|------------|--------|--------------|---------|------|-----------|
| Small (50x50) | 25,000 | Off | 25,000 | 5s | 12 MB |
| Small (50x50) | 25,000 | On | 1,200 | 8s | 600 KB |
| Medium (200x200) | 400,000 | Off | 400,000 | 90s | 200 MB |
| Medium (200x200) | 400,000 | On | 18,000 | 120s | 9 MB |
| Large (500x500) | 2,500,000 | On | 95,000 | 600s | 48 MB |

### Memory Management

- **Batch Processing**: Configurable batch sizes (1,000-20,000 brushes per batch)
- **Automatic Memory Detection**: Adjusts batch size based on available RAM
- **Progressive Writing**: Writes to disk periodically to avoid memory buildup

---

## 🐛 Known Issues & Limitations

### Current Limitations

1. **Non-cube blocks**: Stairs, slabs, fences, etc. are converted as full blocks
2. **Entities**: Chests, signs, item frames are not converted (blocks only)
3. **Biome data**: Not preserved in conversion
4. **Lighting**: Source engine lighting must be configured separately in Hammer

### Workarounds

- **Stairs/Slabs**: Manually adjust in Hammer after conversion
- **Complex structures**: Consider converting in smaller sections
- **Performance**: Use optimization for worlds larger than 100x100 blocks

### Reporting Issues

Found a bug? Please report it on the [Issues](https://github.com/plhappylemonpl/MTS-Minecraft-to-Source/issues) page with:
- MTS version
- Minecraft version
- World size (approximate)
- Error message/logs
- Steps to reproduce

---

## 🗺️ Roadmap

### Planned Features

- [ ] **Multi-block structure support** (stairs, slabs, fences)
- [ ] **Entity conversion** (chests as props, signs as text overlays)
- [ ] **Batch conversion** (multiple worlds at once)
- [ ] **Custom texture pack support**
- [ ] **Automatic selection of textures directly from the texture folder**
- [ ] **Map preview** before conversion
- [ ] **Config file** for advanced settings
- [ ] **Command-line interface** for automation

### Version History

#### v2.0 (Current)
- Complete rewrite in Rust + Tauri
- Greedy mesh optimization
- Block properties support
- Multi-threaded processing
- Modern UI with real-time progress

#### v1.x (Legacy - Python)
- Initial Python implementation
- Basic block conversion
- Simple GUI

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Code Style

- **Rust**: Follow `rustfmt` standards (`cargo fmt`)
- **TypeScript/React**: Follow project ESLint configuration
- **Commits**: Use conventional commit messages

---

## 🙏 Acknowledgments

- **fastanvil** - Efficient Minecraft world reading
- **Tauri** - Modern desktop application framework
- **React** - Reactive UI framework
- **Minecraft community** - Inspiration and support
- **Source engine modding community** - Documentation and tools

---

## 📞 Contact & Support

- **GitHub Issues**: [Report bugs or request features](https://github.com/plhappylemonpl/MTS-Minecraft-to-Source/issues)
- **Discussions**: [Join community discussions](https://github.com/plhappylemonpl/MTS-Minecraft-to-Source/discussions)

---

<div align="center">

**Made with ❤️ by [plhappylemonpl](https://github.com/plhappylemonpl) & [NVTMRE](https://github.com/nvtmre)**

⭐ Star this repository if you find it useful!

</div>
