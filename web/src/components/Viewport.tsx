import React, { useRef, useEffect } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { GeometryResponse } from '@/types/geometry';

interface ViewportProps {
  data: GeometryResponse | null;
}

export const Viewport: React.FC<ViewportProps> = ({ data }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const meshRef = useRef<THREE.Mesh | null>(null);
  
  const raycasterRef = useRef<THREE.Raycaster>(new THREE.Raycaster());
  const mouseRef = useRef<THREE.Vector2>(new THREE.Vector2());
  const hoveredFaceIdRef = useRef<number | null>(null);
  const selectedFaceIdRef = useRef<number | null>(null);

  const baseColor = new THREE.Color('#8899AA');
  const hoverColor = new THREE.Color('#AABBCC');
  const selectColor = new THREE.Color('#FFAA00');

  useEffect(() => {
    if (!containerRef.current) return;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color('#1e1e24');
    sceneRef.current = scene;

    const camera = new THREE.PerspectiveCamera(45, containerRef.current.clientWidth / containerRef.current.clientHeight, 0.1, 1000);
    camera.position.set(20, 20, 20);
    camera.lookAt(0, 0, 0);
    cameraRef.current = camera;

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(containerRef.current.clientWidth, containerRef.current.clientHeight);
    renderer.setPixelRatio(window.devicePixelRatio);
    containerRef.current.appendChild(renderer.domElement);
    rendererRef.current = renderer;

    const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
    scene.add(ambientLight);
    
    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
    directionalLight.position.set(10, 20, 10);
    scene.add(directionalLight);

    const gridHelper = new THREE.GridHelper(50, 50, '#3f3f4e', '#2b2b36');
    scene.add(gridHelper);
    
    const axesHelper = new THREE.AxesHelper(5);
    scene.add(axesHelper);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;

    let animationId: number;
    const animate = () => {
      animationId = requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    };
    animate();

    const handleResize = () => {
      if (!containerRef.current || !camera || !renderer) return;
      camera.aspect = containerRef.current.clientWidth / containerRef.current.clientHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(containerRef.current.clientWidth, containerRef.current.clientHeight);
    };
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      cancelAnimationFrame(animationId);
      controls.dispose();
      renderer.dispose();
      if (containerRef.current && renderer.domElement.parentNode) {
        containerRef.current.removeChild(renderer.domElement);
      }
    };
  }, []);

  useEffect(() => {
    if (!sceneRef.current || !data) return;
    
    const scene = sceneRef.current;
    
    if (meshRef.current) {
      scene.remove(meshRef.current);
      meshRef.current.geometry.dispose();
      (meshRef.current.material as THREE.Material).dispose();
      meshRef.current = null;
    }

    const { positions, normals, indices, face_ids } = data.mesh;

    // De-index the mesh so each triangle has its own vertices.
    // This lets us assign per-vertex face_ids and per-vertex colors
    // without shared-vertex conflicts across different faces.
    const triCount = indices.length / 3;
    const dePositions = new Float32Array(triCount * 9);
    const deNormals = new Float32Array(triCount * 9);
    const deFaceIds = new Int32Array(triCount * 3);
    const colors = new Float32Array(triCount * 9);

    for (let t = 0; t < triCount; t++) {
      const faceId = face_ids[t];
      for (let v = 0; v < 3; v++) {
        const srcIdx = indices[t * 3 + v];
        const dstIdx = t * 3 + v;
        dePositions[dstIdx * 3]     = positions[srcIdx * 3];
        dePositions[dstIdx * 3 + 1] = positions[srcIdx * 3 + 1];
        dePositions[dstIdx * 3 + 2] = positions[srcIdx * 3 + 2];
        deNormals[dstIdx * 3]     = normals[srcIdx * 3];
        deNormals[dstIdx * 3 + 1] = normals[srcIdx * 3 + 1];
        deNormals[dstIdx * 3 + 2] = normals[srcIdx * 3 + 2];
        deFaceIds[dstIdx] = faceId;
        colors[dstIdx * 3]     = baseColor.r;
        colors[dstIdx * 3 + 1] = baseColor.g;
        colors[dstIdx * 3 + 2] = baseColor.b;
      }
    }

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(dePositions, 3));
    geometry.setAttribute('normal', new THREE.Float32BufferAttribute(deNormals, 3));
    geometry.setAttribute('color', new THREE.Float32BufferAttribute(colors, 3));
    geometry.setAttribute('face_ids', new THREE.Int32BufferAttribute(deFaceIds, 1));

    const material = new THREE.MeshPhongMaterial({
      vertexColors: true,
      side: THREE.DoubleSide,
      flatShading: false,
    });

    const mesh = new THREE.Mesh(geometry, material);
    scene.add(mesh);
    meshRef.current = mesh;

    hoveredFaceIdRef.current = null;
    selectedFaceIdRef.current = null;
  }, [data, baseColor]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const updateColors = () => {
      if (!meshRef.current) return;
      
      const geometry = meshRef.current.geometry;
      const faceIds = geometry.getAttribute('face_ids');
      const colorAttr = geometry.getAttribute('color');
      
      if (!faceIds || !colorAttr) return;

      for (let i = 0; i < faceIds.count; i++) {
        const fId = faceIds.getX(i);
        let color = baseColor;
        
        if (fId === selectedFaceIdRef.current) {
          color = selectColor;
        } else if (fId === hoveredFaceIdRef.current) {
          color = hoverColor;
        }
        
        colorAttr.setXYZ(i, color.r, color.g, color.b);
      }
      colorAttr.needsUpdate = true;
    };

    const handlePointerMove = (event: PointerEvent) => {
      if (!container || !cameraRef.current || !meshRef.current) return;
      
      const rect = container.getBoundingClientRect();
      mouseRef.current.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      mouseRef.current.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

      raycasterRef.current.setFromCamera(mouseRef.current, cameraRef.current);
      const intersects = raycasterRef.current.intersectObject(meshRef.current);

      let newHoverId = null;
      if (intersects.length > 0 && intersects[0].faceIndex != null) {
        const triIndex = intersects[0].faceIndex;
        const faceIds = meshRef.current.geometry.getAttribute('face_ids');
        if (faceIds) {
          // De-indexed geometry: vertex index = triIndex * 3
          newHoverId = faceIds.getX(triIndex * 3);
        }
      }

      if (hoveredFaceIdRef.current !== newHoverId) {
        hoveredFaceIdRef.current = newHoverId;
        updateColors();
      }
    };

    const handlePointerDown = (event: PointerEvent) => {
      if (event.button !== 0) return;
      selectedFaceIdRef.current = hoveredFaceIdRef.current;
      updateColors();
    };

    container.addEventListener('pointermove', handlePointerMove);
    container.addEventListener('pointerdown', handlePointerDown);

    return () => {
      container.removeEventListener('pointermove', handlePointerMove);
      container.removeEventListener('pointerdown', handlePointerDown);
    };
  }, [baseColor, hoverColor, selectColor]);

  return (
    <div 
      ref={containerRef} 
      className="viewport-container" 
      data-testid="viewport"
      style={{ width: '100%', height: '100%' }}
    />
  );
};
