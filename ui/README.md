```bash
src/
  routes/
    __root.tsx
    _layout.tsx

    dashboard/
      index.tsx

    services/
      _layout.tsx
      index.tsx
      $serviceId/
        _layout.tsx
        index.tsx

        traces/
          index.tsx
          $traceId.tsx

        logs/
          index.tsx
          $logId.tsx

        metrics/
          index.tsx
          explore.tsx

    settings/
      index.tsx
      appearance.tsx
      storage.tsx
      advanced.tsx


  features/
    services/
      api/
        getServices.ts
        getServiceDetails.ts
      components/
        ServiceList.tsx
        ServiceSelector.tsx
        ServiceHeader.tsx
      hooks/
        useServices.ts
        useService.ts
      utils/
        serviceFormatters.ts
      types/
        service.ts

    traces/
      api/
        getTraces.ts
        getTraceById.ts
      components/
        TraceList.tsx
        TraceTimeline.tsx
        TraceDetailSidebar.tsx
      hooks/
        useTraces.ts
        useTrace.ts
      utils/
        traceFormatters.ts
      types/
        trace.ts

    logs/
      api/
        getLogs.ts
        getLogById.ts
      components/
        LogTable.tsx
        LogDetail.tsx
      hooks/
        useLogs.ts
      utils/
        logFilters.ts
      types/
        log.ts

    metrics/
      api/
        getMetrics.ts
      components/
        MetricChart.tsx
        MetricExplorer.tsx
      hooks/
        useMetrics.ts
      utils/
        metricQueries.ts
      types/
        metric.ts


  components/
    ui/
      Button.tsx
      Card.tsx
      Table.tsx
      Sidebar.tsx
      Topbar.tsx
      Skeleton.tsx
      Dialog.tsx

    layout/
      AppShell.tsx
      MainSidebar.tsx
      ServiceSidebar.tsx


  lib/
    glintClient.ts          // cliente HTTP/gRPC para o binário local
    storage.ts              // helpers para acesso ao DB local
    queryClient.ts          // TanStack Query config

  types/
    common.ts

  styles/
    globals.css
```
