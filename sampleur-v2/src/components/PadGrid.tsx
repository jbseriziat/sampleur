import { usePadStore } from '../store/usePadStore';
import { Pad } from './Pad';

export function PadGrid() {
  const { pads, gridSize } = usePadStore();

  // Padawan mode: 4x4 using pads 0-3, 8-11, 16-19, 24-27 (first 4 of each row of 8)
  const visiblePads = gridSize === 64
    ? pads
    : pads.filter((_, i) => {
        const row = Math.floor(i / 8);
        const col = i % 8;
        return row < 4 && col < 4;
      });

  const cols = gridSize === 64 ? 8 : 4;

  return (
    <div
      className="p-3 flex-1"
      style={{
        display: 'grid',
        gridTemplateColumns: `repeat(${cols}, 1fr)`,
        gap: '6px',
      }}
    >
      {visiblePads.map((pad) => (
        <Pad
          key={pad.id}
          pad={pad}
          size={gridSize === 64 ? 'small' : 'large'}
        />
      ))}
    </div>
  );
}
