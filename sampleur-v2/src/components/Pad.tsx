import { invoke } from '@tauri-apps/api/core';
import { usePadStore } from '../store/usePadStore';
import { PadState } from '../types';

interface PadProps {
  pad: PadState;
  size?: 'small' | 'large';
  // ── Drag & drop props (provided by PadGrid) ────────────────────────────────
  isDragSource?: boolean;
  isDragTarget?: boolean;
  onDragStart?: () => void;
  onDragEnd?: () => void;
  onDragOver?: () => void;
  onDragLeave?: () => void;
  onDrop?: () => void;
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
  onDragStart,
  onDragEnd,
  onDragOver,
  onDragLeave,
  onDrop,
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
    'relative overflow-hidden rounded cursor-pointer select-none',
    'transition-all duration-75 active:scale-95',
    'flex flex-col items-center justify-center gap-0',
    'border-2 font-bold',
    isSmall ? 'h-10 text-xs' : 'h-16 sm:h-20 text-sm',
    pad.hasSample
      ? `${pad.color.tw} ${pad.color.twText}`
      : 'bg-slate-800 text-slate-600 border-slate-700',
    // Selection ring (only when not drag-target to avoid stacking rings)
    isSelected && !isDragTarget ? 'border-white ring-2 ring-white' : 'border-transparent',
    pad.isPlaying ? 'brightness-110' : '',
    !pad.hasSample ? 'opacity-60' : '',
    // Drag visual feedback
    isDragSource ? 'opacity-40 scale-95' : '',
    isDragTarget ? 'ring-2 ring-yellow-400 brightness-125 border-yellow-400' : '',
  ].join(' ');

  // Truncate the file name (no extension) to fit the pad
  const shortName = pad.hasSample && pad.fileName
    ? pad.fileName.replace(/\.[^.]+$/, '').slice(0, isSmall ? 8 : 14)
    : null;

  return (
    <div
      className={baseClasses}
      // ── Playback / selection events ─────────────────────────────────────────
      onMouseDown={handlePress}
      onMouseUp={handleRelease}
      onMouseLeave={handleRelease}
      onTouchStart={(e) => { e.preventDefault(); handlePress(); }}
      onTouchEnd={(e) => { e.preventDefault(); handleRelease(); }}
      // ── HTML5 Drag & Drop (only active in edit mode) ────────────────────────
      draggable={editMode}
      onDragStart={(e) => {
        e.dataTransfer.effectAllowed = 'move';
        onDragStart?.();
      }}
      onDragEnd={() => onDragEnd?.()}
      onDragOver={(e) => {
        e.preventDefault(); // required to allow drop
        e.dataTransfer.dropEffect = 'move';
        onDragOver?.();
      }}
      onDragLeave={() => onDragLeave?.()}
      onDrop={(e) => {
        e.preventDefault();
        onDrop?.();
      }}
    >
      {/* ── Line 1 : custom label (bold) ───────────────────────────── */}
      <span className="z-10 leading-none font-bold drop-shadow px-1 truncate max-w-full">
        {pad.label}
      </span>

      {/* ── Line 2 : file name (regular weight, lighter) ───────────── */}
      {shortName && !isSmall && (
        <span className="z-10 text-[9px] font-normal opacity-70 truncate max-w-full px-1 leading-tight">
          {shortName}
        </span>
      )}

      {/* ── Mode icon (larger & bolder than before) ─────────────────── */}
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

      {/* ── Progress bar ─────────────────────────────────────────────── */}
      {pad.isPlaying && (
        <div
          className="absolute bottom-0 left-0 h-1 bg-white opacity-70 transition-none"
          style={{ width: `${pad.progress * 100}%` }}
        />
      )}

      {/* ── Playing overlay ───────────────────────────────────────────── */}
      {pad.isPlaying && (
        <div className="absolute inset-0 bg-white opacity-10 pointer-events-none" />
      )}

      {/* ── Drag target overlay ───────────────────────────────────────── */}
      {isDragTarget && (
        <div className="absolute inset-0 bg-yellow-400 opacity-15 pointer-events-none rounded" />
      )}
    </div>
  );
}
