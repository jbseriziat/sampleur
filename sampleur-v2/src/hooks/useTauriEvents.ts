import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { usePadStore } from '../store/usePadStore';
import { useMidiStore } from '../store/useMidiStore';

export function useTauriEvents() {
  const { updateProgress } = usePadStore();
  const { setStatus, setLastNote, stopLearn } = useMidiStore();
  const updatePad = usePadStore((s) => s.updatePad);

  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    // Playback progress (~30fps)
    listen<Array<{ pad_id: number; progress: number; is_playing: boolean }>>('playback-progress', (event) => {
      updateProgress(event.payload.map((p) => ({
        padId: p.pad_id,
        progress: p.progress,
        isPlaying: p.is_playing,
      })));
    }).then((fn) => unlisteners.push(fn));

    // Sample loaded
    listen<{ pad_id: number; file_name: string; duration_secs: number }>('sample-loaded', (event) => {
      updatePad(event.payload.pad_id, {
        hasSample: true,
        fileName: event.payload.file_name,
        durationSecs: event.payload.duration_secs,
      });
    }).then((fn) => unlisteners.push(fn));

    // MIDI events
    listen<{ note: number; velocity: number }>('midi-note-received', (event) => {
      setLastNote(event.payload.note);
    }).then((fn) => unlisteners.push(fn));

    listen<{ padId: number; note: number }>('midi-learn-complete', (event) => {
      updatePad(event.payload.padId, { midiNote: event.payload.note });
      stopLearn();
    }).then((fn) => unlisteners.push(fn));

    listen<{ status: string; message?: string }>('midi-status', (event) => {
      setStatus(
        event.payload.status as 'disconnected' | 'connected' | 'error',
        event.payload.message,
      );
    }).then((fn) => unlisteners.push(fn));

    // Launchpad default mapping applied — sync midiNote for every pad in the store
    listen<{ mapping: Array<{ padId: number; note: number }> }>(
      'launchpad-mapping-applied',
      (event) => {
        for (const { padId, note } of event.payload.mapping) {
          updatePad(padId, { midiNote: note });
        }
      },
    ).then((fn) => unlisteners.push(fn));

    return () => { unlisteners.forEach((fn) => fn()); };
  }, []);
}
