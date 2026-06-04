<script setup lang="ts">
import { Gamepad2, Circle, FileCode2, CalendarClock, Settings, Monitor, Gauge, ScanText, RadioTower } from '@lucide/vue'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog'
import { ScrollArea } from '@/components/ui/scroll-area'

const emit = defineEmits<{
  close: []
}>()
</script>

<template>
  <Dialog :open="true" @update:open="(v) => { if (!v) emit('close') }">
    <DialogContent class="sm:max-w-[580px] max-h-[85vh]">
      <DialogHeader>
        <DialogTitle>使用说明</DialogTitle>
        <DialogDescription>
          AutoController 的完整功能指南与快速上手教程
        </DialogDescription>
      </DialogHeader>
      <ScrollArea class="max-h-[65vh] pr-4">
        <div class="flex flex-col gap-4">
          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <Monitor :size="18" />
              <h3 class="text-sm font-semibold text-primary">设备监控</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">点击「添加设备」创建虚拟 Xbox 360 手柄</li>
              <li class="text-sm text-muted-foreground leading-relaxed">点击设备卡片可选中并查看实时输入状态</li>
              <li class="text-sm text-muted-foreground leading-relaxed">选中设备后可通过摇杆可视化和扳机条查看手柄状态</li>
              <li class="text-sm text-muted-foreground leading-relaxed">最多可同时创建 8 个虚拟设备</li>
              <li class="text-sm text-muted-foreground leading-relaxed">设备编号从 0 开始，脚本中可用数字编号引用设备</li>
              <li class="text-sm text-muted-foreground leading-relaxed">绿色图标 = ViGEmBus 已连接（系统可识别），黄色图标 = 模拟模式</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <RadioTower :size="18" />
              <h3 class="text-sm font-semibold text-primary">ViGEmBus 内置驱动</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">当前版本已内置 ViGEmBus 相关运行组件，无需手动下载驱动或复制 ViGEmClient.dll</li>
              <li class="text-sm text-muted-foreground leading-relaxed">首次启动后进入「设备监控」，程序会自动检测并连接虚拟手柄内核驱动</li>
              <li class="text-sm text-muted-foreground leading-relaxed">如果状态显示未连接，点击「尝试热重连并激活驱动」即可重新初始化驱动连接</li>
              <li class="text-sm text-muted-foreground leading-relaxed">绿色图标表示系统可识别虚拟手柄；黄色图标表示当前处于模拟模式，脚本仍可运行但系统/游戏可能无法识别为真实手柄</li>
              <li class="text-sm text-muted-foreground leading-relaxed">若热重连仍失败，优先尝试以管理员身份运行程序并重启应用</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <Gamepad2 :size="18" />
              <h3 class="text-sm font-semibold text-primary">宏控制</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">输入宏名称后点击「开始录制」记录手柄操作</li>
              <li class="text-sm text-muted-foreground leading-relaxed">录制完成后点击「停止录制」保存宏</li>
              <li class="text-sm text-muted-foreground leading-relaxed">可通过回放速度滑块调整播放速率（50%~200%）</li>
              <li class="text-sm text-muted-foreground leading-relaxed">设置循环次数可让宏重复执行</li>
              <li class="text-sm text-muted-foreground leading-relaxed">点击播放按钮回放宏，点击删除按钮移除宏</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <FileCode2 :size="18" />
              <h3 class="text-sm font-semibold text-primary">脚本编辑器</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">左侧面板管理脚本列表，输入名称后点击「新建」创建脚本</li>
              <li class="text-sm text-muted-foreground leading-relaxed">中间区域为代码编辑器，支持手柄控制和延时等 API</li>
              <li class="text-sm text-muted-foreground leading-relaxed">右侧面板提供 API 参考文档</li>
              <li class="text-sm text-muted-foreground leading-relaxed">点击「保存」保存脚本，点击「运行」执行当前脚本</li>
              <li class="text-sm text-muted-foreground leading-relaxed">脚本 API 示例：<code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">press("A")</code> 按下按键，<code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">sleep(1000)</code> 等待 1 秒</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <ScanText :size="18" />
              <h3 class="text-sm font-semibold text-primary">OCR 屏幕识别</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">在「参数配置」的「OCR 自动化配置」中选择识别引擎：Windows 原生 OCR 或内置 PaddleOCR</li>
              <li class="text-sm text-muted-foreground leading-relaxed">点击「添加标定区」打开悬浮框，在屏幕上拖拽选择需要识别的文字区域</li>
              <li class="text-sm text-muted-foreground leading-relaxed">标定后脚本可通过 <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">ocr()</code> 读取默认区域，或通过 <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">ocr(2)</code> 读取指定编号区域</li>
              <li class="text-sm text-muted-foreground leading-relaxed">也可以直接调用 <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">ocr(x, y, w, h)</code> 识别指定屏幕坐标区域</li>
              <li class="text-sm text-muted-foreground leading-relaxed">识别结果会自动过滤空格和换行，适合配合 <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">contains("开始")</code> 等条件判断做菜单自动化</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <CalendarClock :size="18" />
              <h3 class="text-sm font-semibold text-primary">任务调度</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">创建定时任务可自动执行宏或脚本</li>
              <li class="text-sm text-muted-foreground leading-relaxed">支持单次执行、每日定时、固定间隔和 Cron 表达式四种调度方式</li>
              <li class="text-sm text-muted-foreground leading-relaxed">可通过开关按钮启用或禁用任务</li>
              <li class="text-sm text-muted-foreground leading-relaxed">每个任务可设置优先级，高优先级任务优先执行</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <Settings :size="18" />
              <h3 class="text-sm font-semibold text-primary">参数配置</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">通用设置：开机自启动、最小化到托盘、日志级别</li>
              <li class="text-sm text-muted-foreground leading-relaxed">Profile 管理：为不同游戏创建配置文件，绑定游戏进程</li>
              <li class="text-sm text-muted-foreground leading-relaxed">配置修改后自动保存</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <Gauge :size="18" />
              <h3 class="text-sm font-semibold text-primary">Forza 遥测</h3>
            </div>
            <ul class="list-disc pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">进入「Forza 遥测」页面后，应用会监听 Forza Data Out UDP 数据并显示实时仪表盘</li>
              <li class="text-sm text-muted-foreground leading-relaxed">默认监听端口为 <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">20440</code>；如需修改，可点击右上角设置按钮调整 UDP 接收端口，重启后生效</li>
              <li class="text-sm text-muted-foreground leading-relaxed">在 Forza 游戏设置中开启 Data Out，并将 IP 指向运行本程序的电脑；本机运行时可填写 <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">127.0.0.1</code></li>
              <li class="text-sm text-muted-foreground leading-relaxed">页面顶部会显示连接状态、车辆名称、等级、PI 和驱动形式；右上角可切换轮胎数据、打开历史会话和调整设置</li>
              <li class="text-sm text-muted-foreground leading-relaxed">启用自动记录后，可在历史会话中查看圈速、最佳圈和回放数据</li>
            </ul>
          </section>

          <section class="rounded-lg border border-border bg-card p-4">
            <div class="flex items-center gap-2 mb-2 text-primary">
              <Circle :size="18" />
              <h3 class="text-sm font-semibold text-primary">快速上手</h3>
            </div>
            <ol class="list-decimal pl-5 flex flex-col gap-1">
              <li class="text-sm text-muted-foreground leading-relaxed">打开「设备监控」确认 ViGEmBus 内置驱动已连接，必要时点击热重连</li>
              <li class="text-sm text-muted-foreground leading-relaxed">在「设备监控」页面添加虚拟手柄</li>
              <li class="text-sm text-muted-foreground leading-relaxed">在「宏控制」页面录制手柄操作，或在「脚本编辑器」编写自动化脚本</li>
              <li class="text-sm text-muted-foreground leading-relaxed">需要识别屏幕文字时，先在「参数配置」标定 OCR 区域，再在脚本中调用 <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs text-foreground">ocr()</code></li>
              <li class="text-sm text-muted-foreground leading-relaxed">使用「任务调度」设置定时自动执行</li>
              <li class="text-sm text-muted-foreground leading-relaxed">在「日志查看」页面排查问题</li>
            </ol>
          </section>
        </div>
      </ScrollArea>
    </DialogContent>
  </Dialog>
</template>
