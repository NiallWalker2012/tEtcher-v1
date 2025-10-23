# tEtcher 
tEtcher is a terminal based ISO flasher designed to have the speed of cli commands like dd, but also an easier UI  
It is inspired by well-known ISO flashers like dd and BalenaEtcher


# How to run   
# Dependancies to run
There are a few packages necessary to run tEtcher  
These include:
1. GitHub.cli / gh - The command line GitHub utilities
2. Cargo - To build the project

# How to install GitHub CLI
On Windows, you can install the GithHub command line using "winget install --id GitHub.cli"  
On macOS, you can install using the command "brew install gh"
On Linux (deb distros), you can install using the command "sudo apt install gh"  

After installing, verify using "gh --version". Then login to you GitHub account using "gh auth login"

# How to install Cargo
On Windows, run "winget install Rustlang.Rustup" and verify with cargo --version and rustc --version
On macOS, run "brew install rust" and verify like Windows
On Linux, run "sudo apt install cargo" and verify the same way


# Windows  
1. Open Powershell (not cmd). You can do this by searching 'Powershell' in the Windows app search  
2. Navigate to the directory you want to clone tEtcher into using the cd command e.g. "cd C:\Users\<your windows username>" if you aren't already in it  
3. Clone the repository with "git clone https://github.com/NiallWalker2012/tEtcher". You only need to do this once  
4. Go into the repository with the command "cd tEtcher"  
5. Make the run file executable using the command "chmod +x win_run.bat". You only need to do this once   
6. Execute the file using "Start-Process C:\Users\<windows_username\win_run.bat" for example

# macOS and Linux  
1. Open the terminal
2. Navigate to the directory you wish to clone into using the cd command e.g. "cd ~/GitFiles"
3. Clone the repository with the command "git clone https://github.com/NiallWalker2012/tEtcher". You only need to do this once  
4. Go into the directory of the repository using "cd tEtcher"
5. Make the file executable with the command "chmod +x shell_run.sh". You only need to do this once
6. Run with the command "./shell_run.sh"  


# Dependancies 
As tEtcher is made using the Cargo rust compiler and package manager, you will need to install cargo before running  
In most cases, the terminal will tell you how to install cargo if you just enter cargo / cargo --version  


In the future I will integrate a verification option that verifies that the ISO on your computer matches the flashed image on the USB

