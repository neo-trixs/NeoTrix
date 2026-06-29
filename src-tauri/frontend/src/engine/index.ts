// NeoTrix Animation Engine — public API

export {
  type EasingFn,
  type SpringConfig,
  type Keyframe,
  type AnimState,
  type AnimTarget,
  type FrameRenderer,
  type LerpFn,
  ease,
  lerp,
  lerpColor,
  lerpPath,
  springValue,
  springDuration,
  Timeline,
  stagger,
  FrameLoop,
} from './animation';

export {
  type MorphTarget,
  type SVGPathPoint,
  type Frame,
  parsePath,
  serializePath,
  morphPath,
  morphAttrs,
  drawPath,
  animateElement,
  tweenValue,
} from './svg-morph';

export {
  type FrameSequenceConfig,
  type Frame as SeqFrame,
  type SpriteSheetConfig,
  type ProceduralFrameGenerator,
  FrameSequencePlayer,
  SpriteAnimator,
  interpolateFrames,
  captureFrame,
  generateFrameSequence,
} from './frame-sequence';
