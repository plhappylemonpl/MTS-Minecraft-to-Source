# MTS Minecraft to Source
 An application that converts maps from Minecraft to the Source engine (e.g., for Garry’s Mod, Counter-Strike: Source, Team Fortress 2)


## 🔨 Recommended Tools
For map editing, I recommend **Hammer++** (an improved version of standard Hammer):  
👉 [https://ficool2.github.io/HammerPlusPlus-Website/](https://ficool2.github.io/HammerPlusPlus-Website/)

## ✔️ Supported Version
- Tested with Minecraft **1.21.4**
- Full-block texture support: **~90%**

## 📦 Textures
Download the texture pack: [https://mega.nz/file/CpBgRAJT#i8d3DTKHNDWsEMVzr1dTqCCEGF1r6qFkfrx1yvH2ulc](https://mega.nz/file/CpBgRAJT#i8d3DTKHNDWsEMVzr1dTqCCEGF1r6qFkfrx1yvH2ulc)  

Texture installation path (for Garry's Mod):  
`%%Steam%%\steamapps\common\GarrysMod\garrysmod\materials\mc_1.21.4`

## ⚙️ Requirements
To run from source (`MTS_app.py`):

1. **Visual Studio Build Tools 2022**  
   📥 Download: [https://visualstudio.microsoft.com/downloads/](https://visualstudio.microsoft.com/downloads/)  
   During installation check:
   - Under *Desktop & Mobile*:
     - ☑️ Desktop Development with C++
     - ☑️ .NET desktop build tools
   - Under *Individual Components*:
     - ☑️ C++ 2022 Redistributable Update
     - ☑️ C++ CMake tools for Windows
     - ☑️ MSVC v143 - VS 2022 C++ x64/x86 build tools (Latest)
     - ☑️ C++ Build tools core features
     - ☑️ C++ core features
     - ☑️ Visual Studio SDK Build tools Core
     - ☑️ Windows 10 SDK (10.0.20348.0)
     - ☑️ Windows 11 SDK (10.0.22621.0)
     - ☑️ Windows Universal C Runtime

2. **Install Python Dependencies**  
   ```bash
   pip install tk logging
   ```
   and
   ```bash
   pip install amulet-core 
   ```
   💡 If you get errors with `amulet-core` (happens sometimes!):
   ```bash
   pip install amulet juju
   ```

3. **You're all set!** 🎉  
   In project root folder run:
   ```bash
   python MTS_app.py
   ```

**Linux Notice**  
🐧 Support for Linux WIP: Honestly, I don’t know, because I haven’t tested the app on Linux yet. I have absolutely no idea how compiling the 'amulet' API works on that system. On Windows, you need Visual Studio Build Tools 2022 for compilation, but how does that translate to Linux? Seriously, no clue—I’ll probably test it in the next few days.


## 🔧 Recommended Hammer Settings
For better texture scaling:  
`Tools > Options... > Game Configurations > Default texture scale: 0.1562`

