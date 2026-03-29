import { invoke } from '@tauri-apps/api/core';
import { usePadStore } from '../store/usePadStore';
import { PadState } from '../types';

interface PadProps {
  pad: PadState;
  size?: 'small' | 'large';
  // ── Drag & drop props (provided by PadGrid) ────────────────────────────────
  // Mouse-event based — more reliable than HTML5 DnD in Tauri/WebKit
  isDragSource?: boolean;
  isDragTarget?: boolean;
  onDragMouseDown?: () => void;   // Fired on mousedown when in edit mode
  onDragMouseEnter?: () => void;  // Fired on mouseenter while a drag is in progress
  onDragMouseLeave?: () => void;  // Fired on mouseleave while a drag is in progress
}

// Unicode glyphs for play modes — readable at any scale
const MODE_GLYPH: Record<string, string> = {
  loop:    '∞',
  hold:    '⊙',
  oneshot: '▷',
};

export function Pad({
  pad,
  size = 'large',
  isDragSource = false,
  isDragTarget = false,
  onDragMouseDown,
  onDragMouseEnter,
  onDragMouseLeave,
}: PadProps) {
  const { editMode, selectedPadId, selectPad } = usePadStore();

  const handlePress = async () => {
    if (editMode) {
      selectPad(selectedPadId === pad.id ? null : pad.id);
      return;
    }
    if (!pad.hasSample) return;
    const action = pad.mode === 'loop' ? 'toggle' : 'start';
    await invoke('trigger_pad', { padId: pad.id, action });
  };

  const handleRelease = async () => {
    if (editMode || pad.mode !== 'hold') return;
    if (!pad.hasSample) return;
    await invoke('trigger_pad', { padId: pad.id, action: 'stop' });
  };

  const isSelected = selectedPadId === pad.id;
  const isSmall    = size === 'small';

  const baseClasses = [
    // Layout & shape
    'relative overflow-hidden rounded select-none',
    'transition-colors duration-75',
    'flex flex-col items-center justify-center gap-0',
    'border-2 font-bold',
    // Height: let the CSS grid rows determine the height (gridTemplateRows: repeat(N, 1fr))
    // We only enforce a minimum so pads don't collapse on tiny screens.
    isSmall ? 'min-h-8 text-xs' : 'min-h-12 text-sm',
    // Colour
    pad.hasSample
      ? `${pad.color.tw} ${pad.color.twText}`
      : 'bg-slate-800 text-slate-600 border-slate-700',
    // Selection ring (suppressed when pad is a drag target to avoid ring collision)
    isSelected && !isDragTarget ? 'border-white ring-2 ring-white' : 'border-transparent',
    // Playing brightness
    pad.isPlaying ? 'brightness-110' : '',
    // Empty pad dimming
    !pad.hasSample ? 'opacity-60' : '',
    // Drag visual feedback
    isDragSource ? 'opacity-40 scale-95' : '',
    isDragTarget ? 'ring-2 ring-yellow-400 brightness-125 border-yellow-400' : '',
    // Cursor
    editMode ? 'cursor-grab' : 'cursor-pointer',
  ].join(' ');

  // Truncate the file name (no extension) to fit the pad
  const shortName = pad.hasSample && pad.fileName
    ? pad.fileName.replace(/\.[^.]+$/, '').slice(0, isSmall ? 8 : 14)
    : null;

  return (
    <div
      className={baseClasses}
      // ── Playback / selection ───────────────────────────────────────────────
      onMouseDown={() => {
        onDragMouseDown?.();  // Signal drag start (PadGrid tracks this via ref)
        handlePress();        // Select pad (edit mode) or play (play mode)
      }}
      onMouseUp={handleRelease}
      onMouseEnter={() => onDragMouseEnter?.()}
      onMouseLeave={() => {
        onDragMouseLeave?.();
        handleRelease();
      }}
      onTouchStart={(e) => { e.preventDefault(); handlePress(); }}
      onTouchEnd={(e)   => { e.preventDefault(); handleRelease(); }}
    >
      {/* ── Line 1 : custom label (bold) ────────────────────────────────── */}
      <span className="z-10 leading-none font-bold drop-shadow px-1 truncate max-w-full">
        {pad.label}
      </span>

      {/* ── Line 2 : file name (regular weight, lighter) ────────────────── */}
      {shortName && !isSmall && (
        <span className="z-10 text-[9px] font-normal opacity-70 truncate max-w-full px-1 leading-tight">
          {shortName}
        </span>
      )}

      {/* ── Mode icon ────────────────────────────────────────────────────── */}
      {pad.hasSample && (
        <span
          className={[
            'z-10 leading-none drop-shadow',
            isSmall ? 'text-[10px] opacity-75' : 'text-[13px] opacity-85',
          ].join(' ')}
          title={pad.mode}
        >
          {MODE_GLYPH[pad.mode] ?? '▷'}
        </span>
      )}

      {/* ── Progress bar ─────────────────────────────────────────────────── */}
      {pad.isPlaying && (
        <div
          className="absolute bottom-0 left-0 h-1 bg-white opacity-70 transition-none"
          style={{ width: `${pad.progress * 100}%` }}
        />
      )}

      {/* ── Playing overlay ──────────────────────────────────────────────── */}
      {pad.isPlaying && (
        <div className="absolute inset-0 bg-white opacity-10 pointer-events-none" />
      )}

      {/* ── Drag target highlight ────────────────────────────────────────── */}
      {isDragTarget && (
        <div className="absolute inset-0 bg-yellow-400 opacity-20 pointer-events-none rounded" />
      )}
    </div>
  );
}
