import { invoke } from '@tauri-apps/api/core';
import { usePadStore } from '../store/usePadStore';
import { PadState } from '../types';

interface PadProps {
  pad: PadState;
  size?: 'small' | 'large';
}

export function Pad({ pad, size = 'large' }: PadProps) {
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
  const isSmall = size === 'small';

  const baseClasses = [
    'relative overflow-hidden rounded cursor-pointer select-none',
    'transition-all duration-75 active:scale-95',
    'flex flex-col items-center justify-center',
    'border-2 font-bold',
    isSmall ? 'h-10 text-xs' : 'h-16 sm:h-20 text-sm',
    pad.hasSample
      ? `${pad.color.tw} ${pad.color.twText}`
      : 'bg-slate-800 text-slate-600 border-slate-700',
    isSelected ? 'border-white ring-2 ring-white' : 'border-transparent',
    pad.isPlaying ? 'brightness-110' : '',
    !pad.hasSample ? 'opacity-60' : '',
  ].join(' ');

  return (
    <div
      className={baseClasses}
      onMouseDown={handlePress}
      onMouseUp={handleRelease}
      onMouseLeave={handleRelease}
      onTouchStart={(e) => { e.preventDefault(); handlePress(); }}
      onTouchEnd={(e) => { e.preventDefault(); handleRelease(); }}
    >
      {/* Label */}
      <span className="z-10 leading-none font-bold drop-shadow">{pad.label}</span>
      {pad.hasSample && pad.fileName && (
        <span className="z-10 text-[9px] opacity-80 truncate max-w-full px-1 leading-none">
          {pad.fileName.replace(/\.[^.]+$/, '').slice(0, 12)}
        </span>
      )}

      {/* Mode indicator */}
      {pad.hasSample && (
        <span className="z-10 text-[8px] opacity-60 uppercase">
          {pad.mode === 'loop' ? '\u221e' : pad.mode === 'hold' ? '\u2299' : '\u25b7'}
        </span>
      )}

      {/* Progress bar */}
      {pad.isPlaying && (
        <div
          className="absolute bottom-0 left-0 h-1 bg-white opacity-70 transition-none"
          style={{ width: `${pad.progress * 100}%` }}
        />
      )}

      {/* Playing overlay */}
      {pad.isPlaying && (
        <div className="absolute inset-0 bg-white opacity-10 pointer-events-none" />
      )}
    </div>
  );
}
