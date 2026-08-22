import type { Track } from '@/types';

export type QualityTier = {
  label: string;
  cls: string;
  order: number;
};

export function getQualityTier(t: Pick<Track, 'bitrate' | 'sample_rate'>): QualityTier {
  const br = typeof t.bitrate === 'number' ? t.bitrate : null;
  const sr = typeof t.sample_rate === 'number' ? t.sample_rate : null;

  if ((typeof sr === 'number' && sr >= 96000) || (typeof br === 'number' && br >= 800)) {
    return { label: 'Hi-Res', cls: 'hr', order: 5 };
  }
  if ((typeof br === 'number' && br >= 320) || (typeof sr === 'number' && sr >= 48000)) {
    return { label: 'FLAC', cls: 'flac', order: 4 };
  }
  if (typeof br === 'number' && br >= 192) {
    return { label: 'HQ+', cls: 'hqp', order: 3 };
  }
  if (typeof br === 'number' && br >= 128) {
    return { label: 'HQ', cls: 'hq', order: 2 };
  }
  if (br === null || br === undefined) {
    return { label: '·', cls: 'muted', order: 0 };
  }
  return { label: 'SQ', cls: 'sq', order: 1 };
}

export const qualityPillCSS = `
.quality-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 16px;
  padding: 0 7px;
  margin-left: 8px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.06em;
  line-height: 1;
  vertical-align: middle;
}
.quality-muted { background: transparent; color: rgba(255,255,255,0.25); }
.quality-sq    { background: rgba(255,255,255,0.06); color: rgba(255,255,255,0.55); }
.quality-hq    { background: rgba(0,200,255,0.12); color: #49dcff; border: 1px solid rgba(0,200,255,0.22); }
.quality-hqp   { background: rgba(170,120,255,0.12); color: #c29bff; border: 1px solid rgba(170,120,255,0.22); }
.quality-flac  { background: rgba(70,220,140,0.12); color: #6fe6a5; border: 1px solid rgba(70,220,140,0.22); }
.quality-hr    { background: linear-gradient(90deg, rgba(255,200,80,0.14), rgba(255,230,120,0.16)); color: #ffcf66; border: 1px solid rgba(255,200,80,0.3); }
`;
