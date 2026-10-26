  const onRunPrimaryAction = (actionId?: string, formValues?: Record<string, unknown>): void => {
    const id = actionId || primaryAction?.id
    if (!id) return
    void onInvokeAction(id, formValues)
  }
