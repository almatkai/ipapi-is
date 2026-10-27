export function DetailRuntime({
  root,
  title,
  onBack,
  onRunPrimaryAction,
  onOpenActions,
}: {
  root: ExtensionRuntimeNode
  title: string
  onBack: () => void
  onRunPrimaryAction: () => void
  onOpenActions: () => void
}): JSX.Element {
  const markdown = markdownFromNode(root)
  const assetsPath = typeof root.props?.assetsPath === 'string' ? root.props.assetsPath : ''
  const imageOnly = parseImageOnlyMarkdown(markdown, assetsPath)

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="glass-card mb-2 shrink-0 px-3 py-2">
        <div className="flex items-center gap-2">
          <button type="button" className="btn btn-ghost" onClick={onBack}>
            Back
          </button>
          <div className="text-[12px] font-semibold text-ink-2">{title}</div>
          <div className="ml-auto flex items-center gap-1">
            <button type="button" className="btn btn-ghost" onClick={() => onRunPrimaryAction()}>
              Enter
            </button>
            <button type="button" className="btn btn-ghost" onClick={onOpenActions}>
              Cmd+K
            </button>
          </div>
        </div>
      </div>

      <div className="flex min-h-0 flex-1 overflow-hidden">
        <div className="glass-card min-h-0 flex-1 overflow-y-auto px-4 py-4">
          {imageOnly && imageOnly.alt.toLowerCase().includes('color wheel') ? (
            <ColorWheelDetail image={imageOnly} initialColor={root.props?.initialColor} />
          ) : imageOnly ? (
            <div className="flex h-full min-h-[420px] items-center justify-center">
              <img
                src={imageOnly.src}
                alt={imageOnly.alt}
                className="w-auto max-w-full rounded-tezbar-row object-contain"
                style={{ maxHeight: imageOnly.height ? `${imageOnly.height}px` : '70vh' }}
              />
            </div>
          ) : markdown ? (
            <article className="prose prose-invert max-w-none text-[13px] leading-relaxed">
              <Markdown
                text={markdown}
                className="text-[13px] leading-relaxed"
                imageSrcResolver={(src) => resolveMarkdownImageSrc(src, assetsPath)}
              />
            </article>
          ) : (
            <div className="text-[12px] text-ink-3">No detail content</div>
          )}
        </div>

        {root.metadata ? <MetadataSidebar root={root.metadata} /> : null}
      </div>
    </div>
  )
}
