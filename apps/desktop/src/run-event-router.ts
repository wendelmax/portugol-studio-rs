export interface PendingRunEvent {
  id: number;
  handle: () => void;
}

/** Delivers an active event, queues startup events, and drops all other IDs. */
export function routeRunEvent(
  id: number,
  activeRunId: number | null,
  pending: PendingRunEvent[] | null,
  handle: () => void,
): void {
  if (id === activeRunId) handle();
  else if (pending) pending.push({ id, handle });
}

/** Replays only events emitted by the run whose command just resolved. */
export function replayRunEvents(events: PendingRunEvent[], activeRunId: number): void {
  for (const event of events) {
    if (event.id === activeRunId) event.handle();
  }
}
