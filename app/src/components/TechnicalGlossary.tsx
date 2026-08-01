export function TechnicalGlossary() {
  return (
    <details className="rounded-lg border border-border bg-surface/55 px-3 py-2 text-xs text-text-secondary">
      <summary className="cursor-pointer select-none font-semibold text-text-primary">
        高级技术名词
      </summary>
      <dl className="mt-3 grid gap-2 leading-5 sm:grid-cols-2">
        <Term name="启动配置" technical="Profile / .me3" description="记录本次要加载的 Mod、插件和先后顺序。" />
        <Term name="资源型 Mod" technical="package" description="地图、模型、贴图、文本等由 ME3 覆盖加载的文件。" />
        <Term name="功能插件" technical="native DLL" description="注入游戏进程并提供联机或功能逻辑的 DLL。" />
        <Term name="玩法数据文件" technical="regulation.bin" description="保存装备、服装 ID 和玩法参数；移除后可能影响已有存档选择。" />
        <Term name="联机一致性清单" technical="manifest" description="脱敏比较双方文件内容、插件和加载顺序。" />
        <Term name="队友视角资源" technical="_l parts" description="联机时供其他玩家看到的服装资源副本。" />
      </dl>
    </details>
  );
}

function Term({
  name,
  technical,
  description,
}: {
  name: string;
  technical: string;
  description: string;
}) {
  return (
    <div className="rounded-md bg-elevated/65 px-3 py-2">
      <dt className="font-semibold text-text-primary">
        {name} <span className="font-mono font-normal text-text-muted">· {technical}</span>
      </dt>
      <dd className="mt-0.5 text-text-muted">{description}</dd>
    </div>
  );
}
