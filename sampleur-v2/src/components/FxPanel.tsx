import { invoke } from '@tauri-apps/api/core';
import { useFxStore } from '../store/useFxStore';
import { FxState } from '../types';

interface SliderProps {
  label: string;
  param: keyof FxState;
  min: number;
  max: number;
  step: number;
  value: number;
  displayFormat?: (v: number) => string;
}

function FxSlider({ label, param, min, max, step, value, displayFormat }: SliderProps) {
  const { setFxParam } = useFxStore();

  const handleChange = async (v: number) => {
    setFxParam(param, v);
    const paramMap: Record<string, string> = {
      distortion: 'distortion',
      filterFreq: 'filterFreq',
      filterResonance: 'filterResonance',
      delayTime: 'delayTime',
      delayFeedback: 'delayFeedback',
      delayMix: 'delayMix',
      reverbMix: 'reverbMix',
      gateRate: 'gateRate',
      flangerDepth: 'flangerDepth',
      flangerRate: 'flangerRate',
      masterVolume: 'masterVolume',
    };
    await invoke('set_fx_param', { param: paramMap[param] ?? param, value: v });
  };

  return (
    <div className="flex flex-col gap-0.5">
      <div className="flex justify-between text-xs">
        <span className="text-gray-400 uppercase tracking-wide text-[10px]">{label}</span>
        <span className="text-white font-mono text-[10px]">
          {displayFormat ? displayFormat(value) : value.toFixed(2)}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => handleChange(Number(e.target.value))}
        className="w-full"
      />
    </div>
  );
}

export function FxPanel() {
  const { fx } = useFxStore();

  return (
    <div className="w-48 bg-slate-800 border-r border-slate-700 p-2 overflow-y-auto flex flex-col gap-2">
      <h3 className="text-pink-400 text-xs font-bold uppercase tracking-widest">Master FX</h3>

      <FxSlider
        label="Volume"
        param="masterVolume"
        min={0}
        max={2}
        step={0.01}
        value={fx.masterVolume}
        displayFormat={(v) => `${Math.round(v * 100)}%`}
      />

      <div className="border-t border-slate-700 pt-2">
        <p className="text-slate-500 text-[9px] uppercase mb-1">Filtre</p>
        <FxSlider
          label="Fr\u00e9q"
          param="filterFreq"
          min={100}
          max={20000}
          step={10}
          value={fx.filterFreq}
          displayFormat={(v) => `${Math.round(v)}Hz`}
        />
        <FxSlider
          label="R\u00e9so"
          param="filterResonance"
          min={0.1}
          max={20}
          step={0.1}
          value={fx.filterResonance}
          displayFormat={(v) => `Q${v.toFixed(1)}`}
        />
      </div>

      <div className="border-t border-slate-700 pt-2">
        <p className="text-slate-500 text-[9px] uppercase mb-1">Delay</p>
        <FxSlider
          label="Temps"
          param="delayTime"
          min={0.05}
          max={2}
          step={0.01}
          value={fx.delayTime}
          displayFormat={(v) => `${v.toFixed(2)}s`}
        />
        <FxSlider
          label="Feedback"
          param="delayFeedback"
          min={0}
          max={0.95}
          step={0.01}
          value={fx.delayFeedback}
        />
        <FxSlider
          label="Mix"
          param="delayMix"
          min={0}
          max={1}
          step={0.01}
          value={fx.delayMix}
        />
      </div>

      <div className="border-t border-slate-700 pt-2">
        <p className="text-slate-500 text-[9px] uppercase mb-1">Reverb</p>
        <FxSlider
          label="Mix"
          param="reverbMix"
          min={0}
          max={2}
          step={0.01}
          value={fx.reverbMix}
        />
      </div>

      <div className="border-t border-slate-700 pt-2">
        <p className="text-slate-500 text-[9px] uppercase mb-1">Distortion</p>
        <FxSlider
          label="Drive"
          param="distortion"
          min={0}
          max={100}
          step={1}
          value={fx.distortion}
          displayFormat={(v) => `${Math.round(v)}`}
        />
      </div>

      <div className="border-t border-slate-700 pt-2">
        <p className="text-slate-500 text-[9px] uppercase mb-1">Gate</p>
        <FxSlider
          label="Rate"
          param="gateRate"
          min={0}
          max={12}
          step={0.1}
          value={fx.gateRate}
          displayFormat={(v) => `${v.toFixed(1)}Hz`}
        />
      </div>

      <div className="border-t border-slate-700 pt-2">
        <p className="text-slate-500 text-[9px] uppercase mb-1">Flanger</p>
        <FxSlider
          label="Depth"
          param="flangerDepth"
          min={0}
          max={0.02}
          step={0.001}
          value={fx.flangerDepth}
        />
        <FxSlider
          label="Rate"
          param="flangerRate"
          min={0.1}
          max={5}
          step={0.1}
          value={fx.flangerRate}
          displayFormat={(v) => `${v.toFixed(1)}Hz`}
        />
      </div>
    </div>
  );
}
