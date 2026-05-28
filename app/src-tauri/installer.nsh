!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "正在检测并自动安装 ViGEmBus 手柄内核驱动..."
  
  # 1. 将驱动包释放到系统临时目录 $PLUGINSDIR
  SetOutPath "$PLUGINSDIR"
  File "D:\UGit\AutoController\assets\ViGEmBus_1.22.0_x64_x86_arm64.exe"
  
  # 2. 隐式执行静默安装，/norestart 极其重要，防止安装完驱动后 Windows 强行重启电脑
  ExecWait '"$PLUGINSDIR\ViGEmBus_1.22.0_x64_x86_arm64.exe" /quiet /norestart' $0
  
  # 3. 记录日志
  DetailPrint "ViGEmBus 驱动安装结束，返回值: $0"
  
  # 4. 删除临时文件
  Delete "$PLUGINSDIR\ViGEmBus_1.22.0_x64_x86_arm64.exe"

  # 5. 将 ViGEmClient.dll 释放到安装目录，使生产环境下的软件能顺利与内核驱动进行动态连接
  SetOutPath "$INSTDIR"
  File "D:\UGit\AutoController\app\src-tauri\ViGEmClient.dll"
!macroend
