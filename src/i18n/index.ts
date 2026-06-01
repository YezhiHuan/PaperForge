import type { Language } from "../types";

export const messages = {
  en: {
    app: {
      tagline: "Local-first manuscript workspace",
      researchIde: "Research IDE",
      emptyTitle: "Untitled Paper",
      noAuthors: "No authors yet",
      noJournal: "No journal specified",
      saved: "Saved"
    },
    actions: {
      newProject: "New Project",
      createProject: "Create Paper Project",
      importExisting: "Import Existing",
      export: "Export",
      remove: "Remove",
      edit: "Edit",
      preview: "Preview",
      save: "Save",
      createSection: "Create section",
      addCustomSection: "Add custom section",
      openOutputFolder: "Open output folder",
      openProjectFolder: "Open project folder",
      updateTitle: "Update title",
      initWorkspace: "Initialize workspace",
      literature: "Literature",
      references: "References",
      clearChat: "Clear",
      send: "Send",
      thinking: "Thinking…",
      advancedSkill: "Advanced: Run a built-in Skill"
    },
    dashboard: {
      headline: "Write papers from local folders, citations, evidence, and AI proposals.",
      empty: "No projects yet. Create one to generate workspace structure."
    },
    project: {
      manuscript: "Manuscript",
      references: "References",
      literature: "Literature",
      figures: "Figures",
      data: "Data",
      ai: "Agent",
      attachments: "Attachments",
      outputs: "Outputs",
      settings: "Settings",
      projectInfo: "Project Info",
      newSection: "New Section",
      emptyManuscript: "Empty manuscript",
      mvpFolder: "MVP folder"
    },
    writing: {
      editor: "Editor",
      filePreview: "File Preview",
      fullPreview: "Full Preview",
      fullPreviewEmpty: "No merged draft yet",
      fullPreviewHint: "Create or save a section to see the combined preview here."
    },
    modal: {
      createTitle: "Create paper project",
      paperTitle: "Paper title (optional)",
      author: "Authors (optional)",
      journal: "Target journal (optional)",
      workspaceRoot: "Workspace root (optional)",
      sections: "Manuscript Sections",
      sectionNote: "Default is Empty manuscript. Blank section names are ignored.",
      template: "Template",
      naming: "Section file naming",
      emptySections: "No sections selected. Project will start with an empty manuscript.",
      generate: "Generate local folder structure",
      importTitle: "Import existing project",
      importNote: "Enter a PaperForge project folder path. Existing manuscripts are not overwritten.",
      importFolder: "Import folder"
    },
    tools: {
      ai: "Agent Panel",
      references: "Reference Manager",
      citations: "Citation Tasks",
      literature: "Literature Library",
      claims: "Evidence Claims",
      export: "Export",
      settings: "Settings"
    },
    export: {
      markdownPackage: "Export Markdown Package",
      manifestJson: "Export Manifest JSON",
      projectFolder: "Export Project Folder",
      wordDraft: "Export Word Draft",
      latexProject: "Export LaTeX Project",
      wordSoon: "Requires Pandoc. PaperForge will try automatic install on Windows.",
      latexSoon: "Requires Pandoc. PaperForge will try automatic install on Windows.",
      running: "Export running",
      combinedPreview: "Combined draft preview",
      preparing: "Preparing output"
    },
    settings: {
      language: "Language",
      english: "English",
      chinese: "中文",
      workspaceRoot: "Workspace root",
      defaultMode: "Default mode",
      exportMode: "Export mode",
      colorTheme: "Color theme",
      provider: "Provider",
      baseUrl: "Base URL",
      apiKey: "API key",
      model: "Model",
      citationStyle: "Citation style"
    }
  },
  zh: {
    app: {
      tagline: "本地优先论文写作工作区",
      researchIde: "研究写作 IDE",
      emptyTitle: "未命名论文",
      noAuthors: "暂无作者",
      noJournal: "未指定期刊",
      saved: "已保存"
    },
    actions: {
      newProject: "新建项目",
      createProject: "新建论文项目",
      importExisting: "导入项目",
      export: "导出",
      remove: "移除",
      edit: "编辑",
      preview: "预览",
      save: "保存",
      createSection: "新建章节",
      addCustomSection: "添加自定义章节",
      openOutputFolder: "打开导出文件夹",
      openProjectFolder: "打开项目文件夹",
      updateTitle: "更新标题",
      initWorkspace: "初始化工作区",
      literature: "文献",
      references: "引用文献",
      clearChat: "清空",
      send: "发送",
      thinking: "正在思考…",
      advancedSkill: "高级：运行内置技能"
    },
    dashboard: {
      headline: "从本地文件夹、引用、证据和 AI 建议开始写论文。",
      empty: "还没有项目。新建一个项目来生成工作区结构。"
    },
    project: {
      manuscript: "稿件",
      references: "参考文献",
      literature: "文献",
      figures: "图表",
      data: "数据",
      ai: "Agent",
      attachments: "附件",
      outputs: "导出",
      settings: "设置",
      projectInfo: "项目信息",
      newSection: "新建章节",
      emptyManuscript: "空稿件",
      mvpFolder: "MVP 文件夹"
    },
    writing: {
      editor: "编辑",
      filePreview: "当前文件预览",
      fullPreview: "总体预览",
      fullPreviewEmpty: "暂无合并稿件",
      fullPreviewHint: "新建或保存章节后,合并稿件会显示在这里。"
    },
    modal: {
      createTitle: "新建论文项目",
      paperTitle: "论文标题（可选）",
      author: "作者（可选）",
      journal: "目标期刊（可选）",
      workspaceRoot: "工作区根目录（可选）",
      sections: "论文章节",
      sectionNote: "默认空稿件。空章节名会被忽略。",
      template: "模板",
      naming: "章节文件命名",
      emptySections: "未选择章节。项目将以空稿件开始。",
      generate: "生成本地文件夹结构",
      importTitle: "导入已有项目",
      importNote: "输入 PaperForge 项目文件夹路径。已有稿件不会被覆盖。",
      importFolder: "导入文件夹"
    },
    tools: {
      ai: "Agent 面板",
      references: "参考文献管理",
      citations: "引用任务",
      literature: "文献库",
      claims: "证据声明",
      export: "导出",
      settings: "设置"
    },
    export: {
      markdownPackage: "导出 Markdown 文件夹",
      manifestJson: "导出项目 JSON",
      projectFolder: "导出项目文件夹",
      wordDraft: "导出 Word 草稿",
      latexProject: "导出 LaTeX 项目",
      wordSoon: "需要 Pandoc。Windows 上会尝试自动安装。",
      latexSoon: "需要 Pandoc。Windows 上会尝试自动安装。",
      running: "正在导出",
      combinedPreview: "合并稿预览",
      preparing: "正在准备输出"
    },
    settings: {
      language: "语言",
      english: "English",
      chinese: "中文",
      workspaceRoot: "工作区根目录",
      defaultMode: "默认写作模式",
      exportMode: "导出模式",
      colorTheme: "颜色主题",
      provider: "Provider",
      baseUrl: "Base URL",
      apiKey: "API Key",
      model: "模型",
      citationStyle: "引用格式"
    }
  }
} as const;

type MessageTree = typeof messages.en;
type DotPrefix<T extends string> = T extends "" ? "" : `.${T}`;
type DotKeys<T> = {
  [K in keyof T & string]: T[K] extends Record<string, string> ? `${K}${DotPrefix<keyof T[K] & string>}` : K;
}[keyof T & string];

export type MessageKey = DotKeys<MessageTree>;

export function t(language: Language, key: MessageKey) {
  const source = messages[language] ?? messages.en;
  const fallback = messages.en;
  const [section, item] = key.split(".") as [keyof MessageTree, string];
  const value = (source[section] as Record<string, string> | undefined)?.[item];
  return value ?? (fallback[section] as Record<string, string> | undefined)?.[item] ?? key;
}

export function displayTitle(title: string | undefined, language: Language) {
  const clean = title?.trim();
  if (!clean || clean === "Untitled Paper") return t(language, "app.emptyTitle");
  return clean;
}

export function internalTitle(title: string | undefined) {
  return title?.trim() || "Untitled Paper";
}
