import { createRequire } from 'module';
const req = createRequire(import.meta.url);
try {
  console.log('postprocessing:', req.resolve('@react-three/postprocessing'));
} catch(e) { console.log('postprocessing ERROR:', e.message); }
try {
  console.log('fiber:', req.resolve('@react-three/fiber'));
} catch(e) { console.log('fiber ERROR:', e.message); }
try {
  console.log('drei:', req.resolve('@react-three/drei'));
} catch(e) { console.log('drei ERROR:', e.message); }
try {
  console.log('three:', req.resolve('three'));
} catch(e) { console.log('three ERROR:', e.message); }
