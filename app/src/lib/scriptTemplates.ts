export type ScriptTemplateCategory = '基础' | '控制' | 'OCR' | '遥测' | '流程' | '调试'

export interface ScriptTemplate {
  id: string
  name: string
  category: ScriptTemplateCategory
  summary: string
  code: string
}

export const TEMPLATE_CATEGORIES = ['全部', '基础', '控制', 'OCR', '遥测', '流程', '调试'] as const

export const SCRIPT_TEMPLATES: ScriptTemplate[] = [
  {
    id: 'basic-start-here',
    name: '最小脚本',
    category: '基础',
    summary: '设置默认手柄，输出日志，适合作为新脚本起点',
    code: `// 最小脚本
set_default_device(0);

log("脚本开始");
sleep(500);
log("脚本结束");
`,
  },
  {
    id: 'basic-button-tap',
    name: '单次按键',
    category: '控制',
    summary: '按下、保持、释放一个按钮',
    code: `// 单次按键
set_default_device(0);

press("A");
sleep(120);
release("A");
`,
  },
  {
    id: 'controller-button-loop',
    name: '按键循环',
    category: '控制',
    summary: '按固定节奏重复按键，可调整次数和间隔',
    code: `// 按键循环
set_default_device(0);

let repeat_count = 10;
let hold_ms = 120;
let gap_ms = 800;

for i in 0..repeat_count {
  press("A");
  sleep(hold_ms);
  release("A");
  sleep(gap_ms);
}
`,
  },
  {
    id: 'controller-stick-reset',
    name: '摇杆动作',
    category: '控制',
    summary: '推摇杆、保持、回中，适合改成方向动作',
    code: `// 摇杆动作
set_default_device(0);

set_thumb("LeftX", 1.0);
sleep(500);
set_thumb("LeftX", 0.0);

set_thumb("LeftY", 1.0);
sleep(500);
set_thumb("LeftY", 0.0);
`,
  },
  {
    id: 'controller-trigger-pulse',
    name: '扳机脉冲',
    category: '控制',
    summary: '按压扳机并自动归零',
    code: `// 扳机脉冲
set_default_device(0);

set_trigger("Right", 1.0);
sleep(600);
set_trigger("Right", 0.0);
`,
  },
  {
    id: 'ocr-wait-text',
    name: '等待文本',
    category: 'OCR',
    summary: '轮询标定区，直到识别到指定文字',
    code: `// 等待 OCR 文本
set_default_device(0);

let target = "开始";
let matched = false;

for i in 0..60 {
  let text = ocr(1);
  log("OCR: " + text);

  if text.contains(target) {
    matched = true;
    break;
  }

  sleep(1000);
}

if matched {
  press("A");
  sleep(120);
  release("A");
}
`,
  },
  {
    id: 'ocr-read-number',
    name: '读取数字',
    category: 'OCR',
    summary: '把 OCR 结果转成数字并执行阈值判断',
    code: `// OCR 数字阈值
set_default_device(0);

let text = ocr(1).trim();
let value = text.to_int();

log("读取数值");
log(value);

if value >= 100 {
  press("Y");
  sleep(150);
  release("Y");
}
`,
  },
  {
    id: 'ocr-retry-empty',
    name: '空结果重试',
    category: 'OCR',
    summary: '识别为空时自动重试，减少偶发识别失败',
    code: `// OCR 空结果重试
let result = "";

for i in 0..5 {
  result = ocr(1).trim();

  if !result.is_empty() {
    break;
  }

  sleep(300);
}

log("最终识别结果: " + result);
`,
  },
  {
    id: 'telemetry-threshold-guard',
    name: '遥测阈值守卫',
    category: '遥测',
    summary: '读取实时遥测字段，满足阈值后触发动作',
    code: `// 遥测阈值守卫
set_default_device(0);

for i in 0..120 {
  let tel = get_telemetry();

  if tel.is_race_on && tel.speed_kmh < 8.0 {
    log("速度低于阈值");
    log(tel.speed_kmh);

    press("B");
    sleep(160);
    release("B");
  }

  sleep(500);
}
`,
  },
  {
    id: 'telemetry-snapshot-log',
    name: '遥测快照',
    category: '遥测',
    summary: '打印常用实时字段，方便调试条件判断',
    code: `// 遥测快照
let tel = get_telemetry();

log("名称: " + tel.car_name);
log("速度");
log(tel.speed_kmh);
log("挡位");
log(tel.gear);
log("比赛状态");
log(tel.is_race_on);
`,
  },
  {
    id: 'flow-timeboxed-loop',
    name: '限时循环',
    category: '流程',
    summary: '用固定次数模拟限时轮询，避免无限循环失控',
    code: `// 限时循环
let tick_ms = 500;
let max_ticks = 120;

for i in 0..max_ticks {
  log("tick");

  // 在这里写每次轮询要执行的判断

  sleep(tick_ms);
}
`,
  },
  {
    id: 'flow-helper-function',
    name: '复用函数',
    category: '流程',
    summary: '把常用动作封装成函数，减少重复代码',
    code: `// 复用函数
set_default_device(0);

fn tap(button, hold_ms) {
  press(button);
  sleep(hold_ms);
  release(button);
}

tap("A", 120);
sleep(500);
tap("B", 120);
`,
  },
  {
    id: 'debug-scratchpad',
    name: '调试骨架',
    category: '调试',
    summary: '适合配合断点和单步执行观察流程',
    code: `// 调试骨架
set_default_device(0);

let text = ocr(1);
log("OCR: " + text);

let tel = get_telemetry();
log("遥测名称: " + tel.car_name);
log("遥测速度");
log(tel.speed_kmh);

if text.contains("确定") {
  press("A");
  sleep(100);
  release("A");
}
`,
  },
]
