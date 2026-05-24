import React, { useState, useRef, useEffect } from 'react';
import { Project, Scene, TimelineItem, MediaType, DialogueLine, GenerationStatus, BaseCapsule, MythosContainer, ChapterMarker } from '../types';
import { analyzeScript, generateSceneVideo, generateDialogueAudio, generateSceneImage } from '../services/gemini';
import JSZip from 'jszip';
import { 
  Wand2, Play, Pause, Download, ChevronRight, Video, Mic, Book,
  Music, Layers, Settings, ChevronLeft, Save, Upload, AlertCircle,
  Users, GripHorizontal, Scissors, Trash2, Box, Sparkles, Image as ImageIcon,
  ZoomIn, ZoomOut, PlusSquare, Tag, Hexagon, Film, Eye, Square, Repeat, Repeat1, ArrowRightLeft, FolderDown,
  Zap, Flag
} from 'lucide-react';

interface StudioProps {
  project: Project;
  onUpdate: (project: Project) => void;
  onBack: () => void;
}

const VOICES = ['Fenrir', 'Kore', 'Puck', 'Charon', 'Zephyr'];

// Thematic mapping for the Xyrona Prime characters
const CHARACTER_VOICE_PRESETS: Record<string, string> = {
  'ASTRION': 'Fenrir',    // Cosmic, deep, authoritative
  'TALIA': 'Kore',        // Female, sharp, intelligent
  'SILAS': 'Charon',      // Male, troubled, perhaps deeper or textured
  'ELOWEN': 'Zephyr',     // Nature-bound, calm, ethereal
  'OBRAK': 'Puck',        // Cultist, distinct, maybe lighter/trickster
  'NAVIGATOR': 'Zephyr',  // Intellectual, calm
  'SCIENTIST': 'Charon',  // Analytical
  'PILGRIM 1': 'Puck',
  'PILGRIM 2': 'Kore',
  'VENTURANS': 'Kore'
};

type PlaybackMode = 'once' | 'loop' | 'bounce';

const Collapsible: React.FC<{ title: React.ReactNode; children: React.ReactNode; defaultOpen?: boolean; icon?: React.ReactNode }> = ({ title, children, defaultOpen = false, icon }) => {
  const [isOpen, setIsOpen] = useState(defaultOpen);
  return (
    <div className="border border-neutral-800 rounded-lg overflow-hidden bg-neutral-900/50 mb-3">
      <button 
        onClick={() => setIsOpen(!isOpen)}
        className="w-full px-4 py-3 flex items-center justify-between hover:bg-neutral-800 transition-colors text-left"
      >
        <div className="flex items-center gap-3">
          {icon}
          <span className="font-bold text-sm tracking-wide text-neutral-200 uppercase">{title}</span>
        </div>
        <ChevronRight className={`w-4 h-4 text-neutral-500 transition-transform ${isOpen ? 'rotate-90' : ''}`} />
      </button>
      {isOpen && (
        <div className="p-4 border-t border-neutral-800 bg-neutral-950/50 animate-in fade-in slide-in-from-top-1 duration-200">
          {children}
        </div>
      )}
    </div>
  );
};

const Studio: React.FC<StudioProps> = ({ project, onUpdate, onBack }) => {
  const [activeTab, setActiveTab] = useState<'script' | 'board' | 'voices' | 'assets' | 'blueprints'>('script');
  const [scriptText, setScriptText] = useState(project.scriptContent);
  const [analyzing, setAnalyzing] = useState(false);
  const [status, setStatus] = useState<GenerationStatus | null>(null);
  
  // Timeline State
  const [localTimeline, setLocalTimeline] = useState<TimelineItem[]>(project.timeline);
  const [chapterMarkers, setChapterMarkers] = useState<ChapterMarker[]>(project.chapterMarkers || []);
  const [currentTime, setCurrentTime] = useState(0);
  const [isPlaying, setIsPlaying] = useState(false);
  const [totalDuration, setTotalDuration] = useState(30);
  
  // Playback Control State
  const [playbackMode, setPlaybackMode] = useState<PlaybackMode>('once');
  const [loopStart, setLoopStart] = useState(0);
  const [loopEnd, setLoopEnd] = useState(30);
  const playbackDirection = useRef(1); // 1 for forward, -1 for backward (bounce)

  // Interaction State
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [dragStartX, setDragStartX] = useState(0);
  const [originalStartTime, setOriginalStartTime] = useState(0);
  const timelineRef = useRef<HTMLDivElement>(null);
  const [zoom, setZoom] = useState(20); // pixels per second
  const [selectedClipId, setSelectedClipId] = useState<string | null>(null);

  // Audio Preview State
  const [hoveredAudioId, setHoveredAudioId] = useState<string | null>(null);

  // Scene Navigation
  const [focusedSceneIndex, setFocusedSceneIndex] = useState<number | null>(null);

  // Audio Refs
  const audioRefs = useRef<Record<string, HTMLAudioElement>>({});

  // Sync local timeline when project updates externally (or on init)
  useEffect(() => {
    if (!draggingId) {
      setLocalTimeline(project.timeline);
      setChapterMarkers(project.chapterMarkers || []);
    }
  }, [project.timeline, project.chapterMarkers, draggingId]);

  // Audio Playback Sync
  useEffect(() => {
    localTimeline.forEach(item => {
      if (item.track === 'audio' || item.track === 'music') {
        const audio = audioRefs.current[item.id];
        if (!audio) return;

        const start = item.startTime;
        const end = item.startTime + item.duration;
        
        // --- AUDIO PREVIEW LOGIC ---
        if (hoveredAudioId === item.id && !isPlaying) {
             if (audio.paused) {
                 audio.volume = 0.5; // Lower volume for preview
                 audio.currentTime = 0; // Play from start of clip for preview
                 const playPromise = audio.play();
                 if (playPromise !== undefined) playPromise.catch(() => {});
             }
             return; // Skip timeline sync if previewing
        } else if (hoveredAudioId === null && !isPlaying && item.id !== hoveredAudioId) {
             // Stop previewing when mouse leaves
             if (!audio.paused && currentTime < start) { // prevent stopping if playhead is technically over it but paused? No, simple logic: stop if not playing global.
                 audio.pause();
                 audio.currentTime = 0;
             }
        }

        // --- GLOBAL PLAYBACK LOGIC ---
        // Check if playhead is inside the clip's time range
        if (currentTime >= start && currentTime < end) {
          const seekTime = currentTime - start;
          
          // Ensure audio is loaded enough to play
          if (audio.readyState >= 2) { 
             // Sync: Only seek if drift is significant to prevent stutter
             if (Math.abs(audio.currentTime - seekTime) > 0.25) {
               audio.currentTime = seekTime;
             }

             if (isPlaying && playbackDirection.current === 1) { // Only play audio if moving forward
               // Resume playing if needed
               if (audio.paused) {
                 audio.volume = item.track === 'music' ? 0.3 : 1.0;
                 const playPromise = audio.play();
                 if (playPromise !== undefined) {
                   playPromise.catch(error => {
                     // Auto-play policy or interruption
                     console.warn("Playback prevented:", error);
                   });
                 }
               }
             } else {
               // Pause if global state is paused OR bouncing backwards
               if (!audio.paused) audio.pause();
             }
          }
        } else {
          // Playhead is outside clip range: Stop/Pause
          if (!audio.paused || audio.currentTime !== 0) {
            audio.pause();
            if (currentTime < start) {
                // Reset to beginning if we are before the clip (so it's ready to play)
                audio.currentTime = 0;
            }
          }
        }
      }
    });
  }, [currentTime, isPlaying, localTimeline, hoveredAudioId]);

  // Auto-save script changes
  useEffect(() => {
    const handler = setTimeout(() => {
      if (scriptText !== project.scriptContent) {
        onUpdate({ ...project, scriptContent: scriptText, lastModified: Date.now() });
      }
    }, 1000);
    return () => clearTimeout(handler);
  }, [scriptText]);

  // Recalculate duration automatically based on items
  useEffect(() => {
    if (localTimeline.length > 0) {
      const endTimes = localTimeline.map(t => t.startTime + t.duration);
      const max = Math.max(...endTimes);
      setTotalDuration(Math.max(max + 10, 30));
      // Update Loop End if it was default
      if (loopEnd === 30 && max > 30) setLoopEnd(Math.ceil(max));
    }
  }, [localTimeline]);

  // Playback Loop (Enhanced with Loop/Bounce)
  useEffect(() => {
    let interval: any;
    if (isPlaying) {
      const tickRate = 30; // ms
      let lastTime = Date.now();

      interval = setInterval(() => {
        const now = Date.now();
        const delta = (now - lastTime) / 1000;
        lastTime = now;

        setCurrentTime(prev => {
          let next = prev + (delta * playbackDirection.current);

          // Handle Bounds
          if (playbackMode === 'loop') {
             if (next >= loopEnd) return loopStart;
             if (next < loopStart) return loopStart;
          } else if (playbackMode === 'bounce') {
             if (next >= loopEnd) {
                playbackDirection.current = -1;
                return loopEnd;
             }
             if (next <= loopStart) {
                playbackDirection.current = 1;
                return loopStart;
             }
          } else {
             // Once
             if (next >= totalDuration) {
                setIsPlaying(false);
                return 0;
             }
          }

          return Math.max(0, next);
        });
      }, tickRate);
    }
    return () => clearInterval(interval);
  }, [isPlaying, totalDuration, playbackMode, loopStart, loopEnd]);

  // --- Interaction Logic ---

  const handleStop = () => {
    setIsPlaying(false);
    setCurrentTime(0);
    playbackDirection.current = 1;
  };

  const handleAddMarker = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!timelineRef.current) return;
    const rect = timelineRef.current.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const time = clickX / zoom;
    
    const newMarker: ChapterMarker = {
        id: crypto.randomUUID(),
        time: time,
        label: `Chapter ${chapterMarkers.length + 1}`,
        color: '#F59E0B' // Amber
    };
    
    const updatedMarkers = [...chapterMarkers, newMarker];
    setChapterMarkers(updatedMarkers);
    onUpdate({ ...project, chapterMarkers: updatedMarkers });
  };

  const handleDeleteMarker = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const updatedMarkers = chapterMarkers.filter(m => m.id !== id);
    setChapterMarkers(updatedMarkers);
    onUpdate({ ...project, chapterMarkers: updatedMarkers });
  };

  const handleSceneTransitionChange = (sceneId: string, transition: Scene['transition']) => {
    const updatedScenes = project.scenes.map(s => s.id === sceneId ? { ...s, transition } : s);
    onUpdate({ ...project, scenes: updatedScenes });
  };

  // 1. Moving Existing Clips (Mouse Events)
  const handleDragStart = (e: React.MouseEvent, item: TimelineItem) => {
    e.stopPropagation();
    e.preventDefault();
    setDraggingId(item.id);
    setDragStartX(e.clientX);
    setOriginalStartTime(item.startTime);
    setSelectedClipId(item.id);
  };

  const handleDragMove = (e: React.MouseEvent) => {
    if (draggingId && timelineRef.current) {
      const deltaX = e.clientX - dragStartX;
      const deltaTime = deltaX / zoom;
      const newStartTime = Math.max(0, originalStartTime + deltaTime);
      
      setLocalTimeline(prev => prev.map(t => 
        t.id === draggingId ? { ...t, startTime: newStartTime } : t
      ));
    }
  };

  const handleDragEnd = () => {
    if (draggingId) {
      onUpdate({ ...project, timeline: localTimeline });
      setDraggingId(null);
    }
  };

  // 2. Dragging New Assets (HTML5 DnD)
  const handleAssetDragStart = (e: React.DragEvent, capsule: BaseCapsule) => {
    e.dataTransfer.setData('application/json', JSON.stringify(capsule));
    e.dataTransfer.effectAllowed = 'copy';
  };

  const handleTrackDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  };

  const handleTrackDrop = (e: React.DragEvent, trackType: TimelineItem['track']) => {
    e.preventDefault();
    const data = e.dataTransfer.getData('application/json');
    if (!data) return;

    try {
        const capsule: BaseCapsule = JSON.parse(data);
        const rect = e.currentTarget.getBoundingClientRect();
        // Calculate time based on drop position relative to scroll container
        // We need to account for the container's scrollLeft if it was accessible, 
        // but here we are dropping on a track row. 
        // Best approach: drop relative to the timelineRef or the track element.
        
        // Let's assume drop X is relative to viewport, subtract track left, add scroll.
        // Simplified: The timeline tracks should align with timelineRef.
        const scrollContainer = timelineRef.current?.parentElement?.parentElement?.querySelector('.overflow-x-hidden');
        const scrollLeft = scrollContainer?.scrollLeft || 0;
        
        // Fix: Use the offset relative to the track row itself
        const rowRect = e.currentTarget.getBoundingClientRect();
        const offsetX = (e.clientX - rowRect.left) + scrollLeft - 96; // 96 is header width
        const dropTime = Math.max(0, offsetX / zoom);

        // Determine type
        let type = MediaType.BLANK;
        let src = '';
        
        if (capsule.properties?.src) {
            src = capsule.properties.src;
            if (capsule.kind === 'visual') type = MediaType.VIDEO;
            else if (capsule.kind === 'image') type = MediaType.IMAGE;
            else if (capsule.kind === 'audio') type = MediaType.AUDIO;
        }

        const newItem: TimelineItem = {
        id: crypto.randomUUID(),
        sceneId: 'asset-drop',
        type: type,
        src: src,
        duration: 5, // Default duration
        label: capsule.name,
        track: trackType,
        startTime: dropTime,
        capsuleId: capsule.id
        };

        onUpdate({ ...project, timeline: [...localTimeline, newItem] });
    } catch (err) {
        console.error("Drop failed", err);
    }
  };

  // Helper: Add Asset to Genesis Container
  const addToAssets = (currentProj: Project, type: 'visual' | 'audio' | 'image', url: string, label: string, description: string): Project => {
    let updatedGenesis = { ...currentProj.genesis };
    let assetsContainer = updatedGenesis.mythosContainers.find(c => c.name === "Generated Assets");

    if (!assetsContainer) {
      assetsContainer = {
        id: crypto.randomUUID(),
        name: "Generated Assets",
        description: "AI Generated and Uploaded Assets",
        mythosType: "resource",
        color: "#EAB308", // Yellow
        icon: "✨",
        capsules: [],
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
        maxCapsules: 12,
        properties: {},
        tags: []
      };
      updatedGenesis.mythosContainers = [...updatedGenesis.mythosContainers, assetsContainer];
    }

    const newCapsule: BaseCapsule = {
      id: crypto.randomUUID(),
      name: label,
      description: description,
      kind: type,
      glyphs: [],
      traits: [],
      tags: ["generated"],
      properties: { src: url },
      created: new Date().toISOString(),
      modified: new Date().toISOString(),
      author: "Studio",
      version: "1.0"
    };

    updatedGenesis.mythosContainers = updatedGenesis.mythosContainers.map(c => 
      c.id === assetsContainer!.id 
      ? { ...c, capsules: [newCapsule, ...c.capsules] }
      : c
    );

    return { ...currentProj, genesis: updatedGenesis };
  };

  const handleAnalysis = async () => {
    setAnalyzing(true);
    setStatus({ isGenerating: true, progress: 'Parsing screenplay structure...' });
    try {
      const result = await analyzeScript(scriptText);
      const { scenes, genesis } = result;

      // Initialize character voices if needed
      const uniqueChars = new Set<string>();
      scenes.forEach(s => s.characters.forEach(c => uniqueChars.add(c)));
      
      const newVoices = { ...project.characterVoices };
      uniqueChars.forEach(c => {
        if (!newVoices[c]) {
           const upperChar = c.toUpperCase().trim();
           // Default random assignment
           let assignedVoice = VOICES[Math.floor(Math.random() * VOICES.length)];
           
           // Check for Thematic Presets
           if (CHARACTER_VOICE_PRESETS[upperChar]) {
              assignedVoice = CHARACTER_VOICE_PRESETS[upperChar];
           } else {
              // Fuzzy Matching for unlisted characters
              if (upperChar.includes('GUARD') || upperChar.includes('SOLDIER')) assignedVoice = 'Fenrir';
              else if (upperChar.includes('WOMAN') || upperChar.includes('GIRL')) assignedVoice = 'Kore';
           }

           newVoices[c] = assignedVoice;
        }
      });

      onUpdate({ 
        ...project, 
        scenes, 
        characterVoices: newVoices, 
        genesis: genesis, 
        lastModified: Date.now() 
      });
      setActiveTab('board');
      setStatus(null);
    } catch (e) {
      setStatus({ isGenerating: false, progress: '', error: 'Failed to analyze script. Try again.' });
    } finally {
      setAnalyzing(false);
    }
  };

  const handleDownloadZip = async () => {
    setStatus({ isGenerating: true, progress: 'Initializing Project Archive...' });
    try {
        const zip = new JSZip();
        
        // 1. Screenplay Text
        zip.file("screenplay.txt", project.scriptContent);
        
        // 2. Full Project Transcript (Script + Blueprints + Metadata)
        let fullTranscript = `PROJECT: ${project.name}\n`;
        fullTranscript += `DESCRIPTION: ${project.genesis.description}\n`;
        fullTranscript += `GENRE: ${project.genesis.genre.join(', ')}\n`;
        fullTranscript += `THEMES: ${project.genesis.themes.join(', ')}\n`;
        fullTranscript += `\n============================================================\n`;
        fullTranscript += `SCREENPLAY\n`;
        fullTranscript += `============================================================\n\n`;
        fullTranscript += project.scriptContent;
        fullTranscript += `\n\n============================================================\n`;
        fullTranscript += `BLUEPRINTS & CODEX\n`;
        fullTranscript += `============================================================\n\n`;
        
        project.genesis.mythosContainers.forEach(container => {
            container.capsules.forEach(capsule => {
                if (capsule.properties?.content) {
                    fullTranscript += `--- ${capsule.name.toUpperCase()} ---\n`;
                    fullTranscript += `${capsule.description}\n\n`;
                    fullTranscript += `${capsule.properties.content}\n\n`;
                }
            });
        });
        
        zip.file("full_project_transcript.txt", fullTranscript);
        
        // 3. Project JSON (Full state for re-import)
        zip.file("project.json", JSON.stringify(project, null, 2));
        
        // 3. Asset Manifest
        const assetManifest: any[] = [];
        
        // 4. Assets Folder
        const assetsFolder = zip.folder("assets");
        const mediaFolder = zip.folder("media");
        const voicesFolder = zip.folder("voices");
        const videosFolder = zip.folder("videos");
        
        const uniqueUrls = new Map<string, { name: string, type: string, category: string }>();
        
        // Collect URLs from timeline
        project.timeline.forEach(t => { 
            if (t.src && t.src.startsWith('blob:')) {
                const category = t.track === 'visual' ? (t.type === MediaType.VIDEO ? 'videos' : 'media') : 'voices';
                uniqueUrls.set(t.src, { name: t.label, type: t.type, category });
            } 
        });

        // Collect URLs from Genesis Assets
        project.genesis.mythosContainers.forEach(c => {
            c.capsules.forEach(capsule => {
                if (capsule.properties?.src && capsule.properties.src.startsWith('blob:')) {
                    const category = capsule.kind === 'visual' ? 'videos' : (capsule.kind === 'audio' ? 'voices' : 'media');
                    uniqueUrls.set(capsule.properties.src, { name: capsule.name, type: capsule.kind, category });
                }
            });
        });

        // Fetch and add files
        let count = 0;
        const totalAssets = uniqueUrls.size;
        
        for (const [url, meta] of uniqueUrls.entries()) {
            count++;
            setStatus({ isGenerating: true, progress: `Archiving Asset ${count}/${totalAssets}: ${meta.name}` });
            
            try {
                const response = await fetch(url);
                const blob = await response.blob();
                const ext = blob.type.split('/')[1] || 'dat';
                const safeName = meta.name.replace(/[^a-z0-9]/gi, '_').toLowerCase();
                const filename = `${safeName}_${count}.${ext}`;
                
                let targetFolder = assetsFolder;
                if (meta.category === 'videos') targetFolder = videosFolder;
                else if (meta.category === 'voices') targetFolder = voicesFolder;
                else if (meta.category === 'media') targetFolder = mediaFolder;

                targetFolder?.file(filename, blob);
                
                assetManifest.push({
                    originalName: meta.name,
                    archivePath: `${meta.category}/${filename}`,
                    type: meta.type,
                    mimeType: blob.type,
                    size: blob.size
                });
            } catch (err) {
                console.warn(`Failed to zip asset: ${url}`, err);
            }
        }

        zip.file("asset_manifest.json", JSON.stringify(assetManifest, null, 2));

        // Generate Zip
        setStatus({ isGenerating: true, progress: 'Finalizing Project Archive Compression...' });
        const content = await zip.generateAsync({ type: "blob" }, (metadata) => {
            setStatus({ isGenerating: true, progress: `Compressing: ${Math.round(metadata.percent)}%` });
        });
        
        // Trigger Download
        const downloadUrl = URL.createObjectURL(content);
        const a = document.createElement('a');
        a.href = downloadUrl;
        a.download = `${project.name.replace(/\s+/g, '_')}_Master_Archive.zip`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(downloadUrl);
        
        setStatus(null);
    } catch (e) {
        console.error(e);
        setStatus({ isGenerating: false, progress: '', error: 'Failed to create project archive.' });
    }
  };

  const handleAddPlaceholder = (scene: Scene) => {
    // Find end of current visuals
    const lastVisual = localTimeline
      .filter(t => t.track === 'visual')
      .sort((a, b) => (b.startTime + b.duration) - (a.startTime + a.duration))[0];
    const startTime = lastVisual ? (lastVisual.startTime + lastVisual.duration) : 0;

    const newItem: TimelineItem = {
      id: crypto.randomUUID(),
      sceneId: scene.id,
      type: MediaType.BLANK,
      src: '',
      duration: 5,
      label: `SC ${scene.number} Placeholder`,
      track: 'visual',
      startTime
    };

    onUpdate({ ...project, timeline: [...localTimeline, newItem] });
  };

  const handleGenerateVideo = async (scene: Scene) => {
    setStatus({ isGenerating: true, progress: `Generating video for Scene ${scene.number}...` });
    try {
      const url = await generateSceneVideo(scene.description);
      
      const lastVisual = localTimeline
        .filter(t => t.track === 'visual')
        .sort((a, b) => (b.startTime + b.duration) - (a.startTime + a.duration))[0];
      const startTime = lastVisual ? (lastVisual.startTime + lastVisual.duration) : 0;

      const newItem: TimelineItem = {
        id: crypto.randomUUID(),
        sceneId: scene.id,
        type: MediaType.VIDEO,
        src: url,
        duration: 4, 
        label: `SC ${scene.number} Visual`,
        track: 'visual',
        startTime
      };

      let updatedProject = { ...project, timeline: [...localTimeline, newItem] };
      updatedProject = addToAssets(updatedProject, 'visual', url, `Video: Scene ${scene.number}`, scene.description);

      onUpdate(updatedProject);
      setStatus(null);
    } catch (e: any) {
      setStatus({ isGenerating: false, progress: '', error: e.message || 'Video generation failed.' });
    }
  };

  const handleGenerateImage = async (scene: Scene) => {
    setStatus({ isGenerating: true, progress: `Generating storyboard for Scene ${scene.number}...` });
    try {
      const url = await generateSceneImage(scene.description);
      
      const lastVisual = localTimeline
        .filter(t => t.track === 'visual')
        .sort((a, b) => (b.startTime + b.duration) - (a.startTime + a.duration))[0];
      const startTime = lastVisual ? (lastVisual.startTime + lastVisual.duration) : 0;

      const newItem: TimelineItem = {
        id: crypto.randomUUID(),
        sceneId: scene.id,
        type: MediaType.IMAGE,
        src: url,
        duration: 4, 
        label: `SC ${scene.number} Storyboard`,
        track: 'visual',
        startTime
      };

      let updatedProject = { ...project, timeline: [...localTimeline, newItem] };
      updatedProject = addToAssets(updatedProject, 'image', url, `Storyboard: Scene ${scene.number}`, scene.description);

      onUpdate(updatedProject);
      setStatus(null);
    } catch (e: any) {
      setStatus({ isGenerating: false, progress: '', error: e.message || 'Image generation failed.' });
    }
  };

  const handleTestVoice = async (voice: string) => {
    setStatus({ isGenerating: true, progress: `Testing ${voice} voice...` });
    try {
      const url = await generateDialogueAudio("This is a test of the Quantum Quill voice system.", voice);
      const audio = new Audio(url);
      audio.play();
      setStatus(null);
    } catch (err) {
      console.error(err);
      setStatus({ isGenerating: false, progress: '', error: 'Voice test failed.' });
    }
  };

  const handleGenerateTTS = async (scene: Scene, line: DialogueLine) => {
    setStatus({ isGenerating: true, progress: `Generating voice for ${line.character}...` });
    try {
      const voice = project.characterVoices[line.character] || 'Fenrir';
      const url = await generateDialogueAudio(line.text, voice);
      
      const newItem: TimelineItem = {
        id: crypto.randomUUID(),
        sceneId: scene.id,
        type: MediaType.AUDIO,
        src: url,
        duration: Math.max(2, line.text.length / 15), 
        label: `${line.character}: ${line.text.substring(0, 10)}...`,
        track: 'audio',
        startTime: currentTime
      };

      let updatedProject = { ...project, timeline: [...localTimeline, newItem] };
      updatedProject = addToAssets(updatedProject, 'audio', url, `Voice: ${line.character}`, line.text);

      onUpdate(updatedProject);
      setStatus(null);
    } catch (e: any) {
      setStatus({ isGenerating: false, progress: '', error: e.message || 'Audio generation failed.' });
    }
  };

  const handleGenerateAllDialogue = async () => {
    setStatus({ isGenerating: true, progress: 'Initializing batch audio generation...' });
    let updatedProject = { ...project };
    let currentTimelineOffset = 0;
    
    try {
      // Iterate through scenes
      for (const scene of updatedProject.scenes) {
        // Simple logic: Place audio sequentially. 
        // If visuals exist for this scene, we try to start at the scene start.
        // Otherwise we continue from where the last audio left off.
        
        // Find if scene has existing visual track items to align with
        const sceneVisuals = updatedProject.timeline
             .filter(t => t.sceneId === scene.id && t.track === 'visual')
             .sort((a,b) => a.startTime - b.startTime);
        
        if (sceneVisuals.length > 0) {
            // Align cursor with scene visual start if we jumped scenes
            currentTimelineOffset = Math.max(currentTimelineOffset, sceneVisuals[0].startTime);
        }

        for (const line of scene.dialogue) {
           if (!line.text) continue;

           setStatus({ isGenerating: true, progress: `Generating: ${line.character} (Scene ${scene.number})...` });
           
           const voice = updatedProject.characterVoices[line.character] || 'Fenrir';
           
           // Generate
           const url = await generateDialogueAudio(line.text, voice);
           const duration = Math.max(2, line.text.length / 15); // Estimation
           
           // Create Timeline Item
           const newItem: TimelineItem = {
               id: crypto.randomUUID(),
               sceneId: scene.id,
               type: MediaType.AUDIO,
               src: url,
               duration: duration,
               label: `${line.character}: ${line.text.substring(0, 15)}...`,
               track: 'audio',
               startTime: currentTimelineOffset
           };

           // Add to Timeline locally and to project object
           updatedProject.timeline.push(newItem);
           
           // Add to Assets
           updatedProject = addToAssets(updatedProject, 'audio', url, `Voice: ${line.character} (SC${scene.number})`, line.text);
           
           // Advance cursor
           currentTimelineOffset += duration;
        }
      }
      
      // Final Update
      onUpdate(updatedProject);
      setLocalTimeline(updatedProject.timeline);
      setStatus(null);
    } catch (e: any) {
        console.error(e);
        setStatus({ isGenerating: false, progress: '', error: e.message || 'Batch generation failed.' });
    }
  };

  // --- NEW: Generic Asset Upload for Video/Audio/Images ---
  const handleAssetUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      const file = e.target.files[0];
      const url = URL.createObjectURL(file);
      
      let type: 'visual' | 'audio' | 'image' = 'image'; // default
      let mediaType = MediaType.IMAGE;
      let track: TimelineItem['track'] = 'visual';

      if (file.type.startsWith('video/')) {
         type = 'visual';
         mediaType = MediaType.VIDEO;
         track = 'visual';
      } else if (file.type.startsWith('audio/')) {
         type = 'audio';
         mediaType = MediaType.AUDIO;
         track = 'audio';
      }

      // Add to Assets Library
      let updatedProject = addToAssets(project, type, url, file.name, 'User Uploaded');
      
      onUpdate(updatedProject);
    }
  };

  // Deprecated specific handler in favor of generic one, but kept for soundtrack button compatibility
  const handleSoundtrackUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files && e.target.files[0]) {
        const file = e.target.files[0];
        const url = URL.createObjectURL(file);
        
        // Add to Assets first
        let updatedProject = addToAssets(project, 'audio', url, `Soundtrack: ${file.name}`, 'User Soundtrack');
        
        // Add to Timeline
        const newItem: TimelineItem = {
            id: crypto.randomUUID(),
            sceneId: 'global',
            type: MediaType.AUDIO,
            src: url,
            duration: 20, // Default for music
            label: file.name,
            track: 'music',
            startTime: currentTime
        };
        
        updatedProject = { ...updatedProject, timeline: [...localTimeline, newItem] };
        onUpdate(updatedProject);
      }
  }

  const deleteClip = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const updated = project.timeline.filter(t => t.id !== id);
    onUpdate({ ...project, timeline: updated });
    if (selectedClipId === id) setSelectedClipId(null);
    delete audioRefs.current[id];
  };

  const updateClipDuration = (id: string, newDuration: number) => {
    const updated = project.timeline.map(t => t.id === id ? { ...t, duration: newDuration } : t);
    onUpdate({ ...project, timeline: updated });
  };

  const handleZoom = (direction: 'in' | 'out') => {
    setZoom(prev => {
        const newZoom = direction === 'in' ? prev * 1.2 : prev / 1.2;
        return Math.max(5, Math.min(200, newZoom));
    });
  };

  const getCurrentVisual = () => {
    return localTimeline.find(t => 
      t.track === 'visual' && 
      currentTime >= t.startTime && 
      currentTime < (t.startTime + t.duration)
    );
  };

  const currentVisual = getCurrentVisual();
  const selectedClip = localTimeline.find(t => t.id === selectedClipId);

  // --- NEW: Calculate Scene Regions for Sync Track ---
  const getSceneRegions = () => {
     const regions: { scene: Scene, start: number, end: number }[] = [];
     
     project.scenes.forEach(scene => {
        const items = localTimeline.filter(t => t.sceneId === scene.id);
        if (items.length > 0) {
            const start = Math.min(...items.map(t => t.startTime));
            const end = Math.max(...items.map(t => t.startTime + t.duration));
            regions.push({ scene, start, end });
        }
     });
     
     return regions;
  };

  const sceneRegions = getSceneRegions();

  return (
    <div 
      className="flex flex-col w-full h-full bg-neutral-950"
      onMouseMove={handleDragMove}
      onMouseUp={handleDragEnd}
      onMouseLeave={handleDragEnd}
    >
      
      {/* Top Section */}
      <div className="flex-1 flex overflow-hidden min-h-0">
        
        {/* Left Panel */}
        <div className="w-1/2 flex flex-col border-r border-neutral-800 min-h-0">
          <div className="flex border-b border-neutral-800 bg-neutral-900 shrink-0">
            <button onClick={() => setActiveTab('script')} className={`flex-1 py-3 text-xs font-medium uppercase tracking-wider ${activeTab === 'script' ? 'text-indigo-400 border-b-2 border-indigo-500 bg-indigo-500/10' : 'text-neutral-500 hover:text-white'}`}>Script</button>
            <button onClick={() => setActiveTab('board')} className={`flex-1 py-3 text-xs font-medium uppercase tracking-wider ${activeTab === 'board' ? 'text-indigo-400 border-b-2 border-indigo-500 bg-indigo-500/10' : 'text-neutral-500 hover:text-white'}`}>Scenes</button>
            <button onClick={() => setActiveTab('voices')} className={`flex-1 py-3 text-xs font-medium uppercase tracking-wider ${activeTab === 'voices' ? 'text-indigo-400 border-b-2 border-indigo-500 bg-indigo-500/10' : 'text-neutral-500 hover:text-white'}`}>Voices</button>
            <button onClick={() => setActiveTab('assets')} className={`flex-1 py-3 text-xs font-medium uppercase tracking-wider ${activeTab === 'assets' ? 'text-indigo-400 border-b-2 border-indigo-500 bg-indigo-500/10' : 'text-neutral-500 hover:text-white'}`}>Assets</button>
            <button onClick={() => setActiveTab('blueprints')} className={`flex-1 py-3 text-xs font-medium uppercase tracking-wider ${activeTab === 'blueprints' ? 'text-indigo-400 border-b-2 border-indigo-500 bg-indigo-500/10' : 'text-neutral-500 hover:text-white'}`}>Blueprints</button>
          </div>

          <div className="flex-1 overflow-y-auto bg-neutral-925 relative min-h-0">
            
            {activeTab === 'script' && (
              <div className="p-0 h-full flex flex-col">
                <textarea 
                  value={scriptText}
                  onChange={(e) => setScriptText(e.target.value)}
                  className="flex-1 w-full bg-neutral-950 p-8 text-neutral-300 font-script text-base resize-none focus:outline-none leading-relaxed"
                  placeholder="Paste your screenplay here..."
                  spellCheck={false}
                />
                <div className="p-4 border-t border-neutral-800 bg-neutral-900 shrink-0">
                  <button 
                    onClick={handleAnalysis}
                    disabled={analyzing}
                    className="w-full bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white font-medium py-3 rounded-lg flex items-center justify-center gap-2 transition-all disabled:opacity-50"
                  >
                    {analyzing ? <span className="animate-pulse">Analyzing...</span> : <><Wand2 className="w-4 h-4" /> Analyze Script</>}
                  </button>
                </div>
              </div>
            )}

            {activeTab === 'board' && (
              <div className="p-6 space-y-6 flex flex-col h-full min-h-0">
                {/* Scene Navigator Header */}
                <div className="flex items-center justify-between mb-4 sticky top-0 bg-neutral-925 z-10 pb-2 border-b border-neutral-800 shrink-0">
                    <button 
                        disabled={focusedSceneIndex === null || focusedSceneIndex <= 0}
                        onClick={() => setFocusedSceneIndex(prev => (prev !== null && prev > 0 ? prev - 1 : 0))}
                        className="p-2 hover:bg-neutral-800 rounded disabled:opacity-30"
                    >
                        <ChevronLeft className="w-5 h-5" />
                    </button>
                    <span className="font-bold text-neutral-400 text-sm">
                        {focusedSceneIndex !== null ? `SCENE ${project.scenes[focusedSceneIndex]?.number || 0}` : "ALL SCENES"}
                    </span>
                    <div className="flex gap-2">
                        {focusedSceneIndex !== null && (
                            <button onClick={() => setFocusedSceneIndex(null)} className="text-xs text-indigo-400 hover:underline">
                                Show All
                            </button>
                        )}
                        <button 
                            disabled={focusedSceneIndex === null || focusedSceneIndex >= project.scenes.length - 1}
                            onClick={() => setFocusedSceneIndex(prev => (prev !== null ? prev + 1 : 0))}
                            className="p-2 hover:bg-neutral-800 rounded disabled:opacity-30"
                        >
                            <ChevronRight className="w-5 h-5" />
                        </button>
                    </div>
                </div>

                {project.scenes.length === 0 && (
                  <div className="text-center text-neutral-500 mt-20">
                    <p>No scenes analyzed yet.</p>
                  </div>
                )}

                <div className="flex-1 overflow-y-auto space-y-4 pb-20">
                    {project.scenes.map((scene, index) => {
                        if (focusedSceneIndex !== null && index !== focusedSceneIndex) return null;

                        return (
                          <Collapsible 
                            key={scene.id} 
                            title={`SCENE ${scene.number}: ${scene.heading.substring(0, 30)}...`}
                            icon={<Film className="w-4 h-4 text-indigo-400" />}
                            defaultOpen={focusedSceneIndex !== null}
                          >
                            <div className="space-y-4">
                              <div className="flex justify-between items-center">
                                <div className="text-xs font-mono text-indigo-300">{scene.heading}</div>
                                <div className="flex gap-2">
                                  <div className="flex items-center gap-1 bg-neutral-900 rounded px-2 border border-neutral-800">
                                     <Zap className="w-3 h-3 text-yellow-500" />
                                     <select 
                                        value={scene.transition || 'cut'}
                                        onChange={(e) => handleSceneTransitionChange(scene.id, e.target.value as any)}
                                        className="bg-transparent text-[10px] text-neutral-400 outline-none uppercase font-bold w-16"
                                     >
                                        <option value="cut">Cut</option>
                                        <option value="fade">Fade</option>
                                        <option value="dissolve">Dissolve</option>
                                        <option value="wipe">Wipe</option>
                                     </select>
                                  </div>
                                  <button onClick={() => handleAddPlaceholder(scene)} className="p-1.5 hover:bg-neutral-800 rounded border border-neutral-800" title="Add Placeholder"><PlusSquare className="w-3 h-3" /></button>
                                  <button onClick={() => handleGenerateImage(scene)} className="p-1.5 hover:bg-indigo-900 rounded border border-neutral-800" title="Storyboard"><ImageIcon className="w-3 h-3" /></button>
                                  <button onClick={() => handleGenerateVideo(scene)} className="p-1.5 hover:bg-indigo-900 rounded border border-neutral-800" title="Video"><Video className="w-3 h-3" /></button>
                                </div>
                              </div>
                              <p className="text-neutral-400 text-sm leading-relaxed italic">{scene.description}</p>
                              <div className="space-y-2">
                                {scene.dialogue.map((line) => (
                                  <div key={line.id} className="flex items-center justify-between bg-neutral-900/50 p-2 rounded border border-neutral-800/50">
                                    <div className="text-sm">
                                      <span className="text-indigo-400 font-bold block text-[10px] uppercase">{line.character}</span>
                                      <span className="text-neutral-300">{line.text}</span>
                                    </div>
                                    <button onClick={() => handleGenerateTTS(scene, line)} className="p-1.5 hover:bg-neutral-800 rounded-full text-neutral-500 hover:text-white transition-colors">
                                      <Mic className="w-3 h-3" />
                                    </button>
                                  </div>
                                ))}
                              </div>
                            </div>
                          </Collapsible>
                        );
                    })}
                </div>
              </div>
            )}

            {activeTab === 'voices' && (
              <div className="p-8">
                 <div className="flex items-center justify-between mb-6">
                    <h3 className="text-xl font-display font-bold text-white flex items-center gap-2">
                        <Users className="w-5 h-5" /> Character Voices
                    </h3>
                    <button 
                        onClick={handleGenerateAllDialogue}
                        className="bg-indigo-600 hover:bg-indigo-500 text-white text-xs px-4 py-2 rounded-full flex items-center gap-2 transition-all shadow-lg shadow-indigo-900/20"
                    >
                        <Zap className="w-3 h-3 fill-current" />
                        Generate All Dialogue
                    </button>
                 </div>

                 <div className="space-y-4">
                    {Object.keys(project.characterVoices).length === 0 && (
                        <p className="text-neutral-500 italic">No characters found. Analyze the script first.</p>
                    )}
                    {Object.entries(project.characterVoices).map(([char, voice]) => (
                        <Collapsible key={char} title={char} icon={<Users className="w-4 h-4 text-indigo-400" />}>
                          <div className="flex items-center justify-between gap-4">
                              <div className="flex flex-col gap-1">
                                <span className="text-[10px] text-neutral-500 uppercase font-bold">Assigned Model</span>
                                <select 
                                    value={voice as string}
                                    onChange={(e) => onUpdate({
                                        ...project,
                                        characterVoices: { ...project.characterVoices, [char]: e.target.value }
                                    })}
                                    className="bg-neutral-950 border border-neutral-700 rounded px-3 py-2 text-sm text-white focus:border-indigo-500 focus:outline-none"
                                >
                                    {VOICES.map(v => (
                                        <option key={v} value={v}>{v}</option>
                                    ))}
                                </select>
                              </div>
                              <button 
                                onClick={() => handleTestVoice(voice as string)}
                                className="p-3 bg-neutral-800 hover:bg-neutral-700 rounded-lg text-indigo-400 transition-colors"
                                title="Test Voice"
                              >
                                <Play className="w-4 h-4" />
                              </button>
                          </div>
                        </Collapsible>
                    ))}
                 </div>
              </div>
            )}

            {activeTab === 'blueprints' && (
              <div className="p-6">
                <div className="flex items-center justify-between mb-6">
                  <h3 className="text-xl font-display font-bold text-white flex items-center gap-2">
                    <Book className="w-5 h-5" /> Blueprints & Bible
                  </h3>
                </div>
                
                <div className="space-y-4">
                  {project.genesis.mythosContainers
                    .filter(c => c.mythosType === 'blueprint' || c.name === 'BLUEPRINTS')
                    .map((container) => (
                      <Collapsible 
                        key={container.id} 
                        title={container.name} 
                        icon={<span style={{color: container.color}}>{container.icon}</span>}
                        defaultOpen={true}
                      >
                        <div className="space-y-3">
                          {container.capsules.map((capsule) => (
                            <div 
                              key={capsule.id} 
                              className="bg-neutral-900 border border-neutral-800 rounded-lg p-4"
                            >
                              <div className="flex justify-between items-start mb-3">
                                <div>
                                  <h4 className="font-bold text-white text-base">{capsule.name}</h4>
                                  <p className="text-xs text-neutral-500 mt-1">{capsule.description}</p>
                                </div>
                                <span className="text-[10px] px-2 py-1 rounded bg-neutral-800 text-indigo-400 border border-neutral-700 uppercase font-mono">
                                  {capsule.kind}
                                </span>
                              </div>
                              
                              <div className="bg-black/40 p-4 rounded border border-neutral-800 max-h-[500px] overflow-y-auto custom-scrollbar">
                                <pre className="text-xs text-neutral-300 font-mono whitespace-pre-wrap leading-relaxed">
                                  {capsule.properties?.content}
                                </pre>
                              </div>
                            </div>
                          ))}
                        </div>
                      </Collapsible>
                    ))}
                </div>
              </div>
            )}

            {activeTab === 'assets' && (
              <div className="p-6">
                 <div className="flex items-center justify-between mb-6">
                    <h3 className="text-xl font-display font-bold text-white flex items-center gap-2">
                        <Box className="w-5 h-5" /> Genesis Assets
                    </h3>
                    <div className="flex gap-2">
                        <button onClick={handleDownloadZip} className="bg-neutral-800 hover:bg-indigo-600 text-white text-xs px-3 py-2 rounded flex items-center gap-2 transition-colors border border-neutral-700" title="Export Project as ZIP">
                            <FolderDown className="w-3 h-3" />
                            Export
                        </button>
                        <label className="bg-neutral-800 hover:bg-indigo-600 text-white text-xs px-3 py-2 rounded flex items-center gap-2 cursor-pointer transition-colors border border-neutral-700">
                            <Upload className="w-3 h-3" />
                            Upload Media
                            <input type="file" accept="video/*,audio/*,image/*" className="hidden" onChange={handleAssetUpload} />
                        </label>
                    </div>
                 </div>
                
                {!project.genesis.mythosContainers.length && (
                  <div className="text-center text-neutral-500 mt-10 p-4 border border-dashed border-neutral-800 rounded-lg">
                    <Sparkles className="w-8 h-8 mx-auto mb-2 opacity-50" />
                    <p>No assets generated yet.</p>
                    <p className="text-xs mt-2">Analyze the script to extract Core Data Types.</p>
                  </div>
                )}

                <div className="space-y-4">
                  {project.genesis.mythosContainers.map((container) => (
                    <Collapsible 
                      key={container.id} 
                      title={container.name} 
                      icon={<span style={{color: container.color}}>{container.icon}</span>}
                      defaultOpen={container.name === 'BLUEPRINTS'}
                    >
                      <div className="space-y-3">
                        {container.capsules.map((capsule) => (
                          <div 
                            key={capsule.id} 
                            className="bg-neutral-900 border border-neutral-800 rounded-lg p-3 hover:border-indigo-500/50 transition-all group cursor-grab active:cursor-grabbing relative"
                            draggable={true}
                            onDragStart={(e) => handleAssetDragStart(e, capsule)}
                          >
                            <div className="flex justify-between items-start">
                              <div>
                                <div className="font-bold text-white text-sm flex items-center gap-2">
                                    {capsule.name}
                                    {capsule.tags.some(t => t.includes('CHAR') || t.includes('LOC')) && (
                                        <Hexagon className="w-3 h-3 text-indigo-500 fill-indigo-500/20" />
                                    )}
                                </div>
                                <div className="text-[10px] text-neutral-500 uppercase tracking-wider mt-0.5">
                                    {capsule.kind}
                                </div>
                              </div>
                              <GripHorizontal className="w-4 h-4 text-neutral-600" />
                            </div>
                            
                            {capsule.kind === 'blueprint' ? (
                              <div className="mt-3 bg-black/40 p-3 rounded border border-neutral-800 max-h-60 overflow-y-auto custom-scrollbar">
                                <pre className="text-[11px] text-neutral-400 font-mono whitespace-pre-wrap leading-relaxed">
                                  {capsule.properties?.content}
                                </pre>
                              </div>
                            ) : (
                              <p className="text-xs text-neutral-400 mt-2 line-clamp-2">{capsule.description}</p>
                            )}
                            
                            <div className="flex flex-wrap gap-1 mt-2">
                                {capsule.tags.map(tag => (
                                    <span key={tag} className="text-[9px] px-1.5 py-0.5 rounded bg-neutral-800 text-neutral-500 border border-neutral-700">
                                        {tag}
                                    </span>
                                ))}
                            </div>

                            {capsule.properties?.src && (
                                <div className="mt-2 text-[10px] text-neutral-500 flex items-center justify-between border-t border-neutral-800 pt-2">
                                   <div className="flex items-center gap-1">
                                     {capsule.kind === 'visual' && <Video className="w-3 h-3" />}
                                     {capsule.kind === 'image' && <ImageIcon className="w-3 h-3" />}
                                     {capsule.kind === 'audio' && <Mic className="w-3 h-3" />}
                                     <span>Asset Ready</span>
                                   </div>
                                   <a 
                                     href={capsule.properties.src} 
                                     download={`${capsule.name.replace(/\s+/g, '_')}.${capsule.kind === 'audio' ? 'wav' : 'png'}`}
                                     className="text-indigo-400 hover:text-indigo-300 flex items-center gap-1"
                                     onClick={(e) => e.stopPropagation()}
                                   >
                                     <Download className="w-3 h-3" />
                                     Download
                                   </a>
                                </div>
                            )}
                          </div>
                        ))}
                      </div>
                    </Collapsible>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Right Panel - Preview */}
        <div className="w-1/2 flex flex-col bg-neutral-950 relative min-h-0">
          <div className="flex-1 flex flex-col items-center justify-center p-8 border-b border-neutral-800 bg-black min-h-0">
            <div className="aspect-video w-full max-w-2xl bg-neutral-900 rounded-lg border border-neutral-800 shadow-2xl overflow-hidden relative group">
              {currentVisual ? (
                currentVisual.type === MediaType.VIDEO ? (
                  <video 
                    key={currentVisual.id} 
                    src={currentVisual.src} 
                    className="w-full h-full object-cover"
                    loop={false} // controlled by react loop
                    muted 
                    // Video playback sync
                    ref={(el) => {
                        if (el) {
                            const offset = currentTime - currentVisual.startTime;
                            if (Math.abs(el.currentTime - offset) > 0.5) el.currentTime = offset;
                            if (isPlaying && playbackDirection.current === 1) { // Only play forward
                                if (el.paused) el.play();
                            } else {
                                if (!el.paused) el.pause();
                            }
                        }
                    }}
                  />
                ) : currentVisual.type === MediaType.IMAGE ? (
                   <img 
                    key={currentVisual.id} 
                    src={currentVisual.src} 
                    className="w-full h-full object-cover animate-in fade-in zoom-in-95 duration-1000"
                   />
                ) : (
                  <div className="w-full h-full flex flex-col items-center justify-center bg-indigo-900/10">
                     <div className="text-6xl font-bold text-indigo-800 opacity-20 select-none">
                        {currentVisual.label.includes('SC') ? currentVisual.label.split(' ')[1] : '#'}
                     </div>
                     <p className="text-indigo-300 font-bold mt-4 z-10">{currentVisual.label}</p>
                  </div>
                )
              ) : (
                 <div className="w-full h-full flex items-center justify-center text-neutral-600 flex-col gap-2">
                    <div className="w-12 h-12 rounded-full border-2 border-neutral-700 flex items-center justify-center">
                      <Play className="w-5 h-5 ml-1" />
                    </div>
                    <p className="text-xs uppercase tracking-widest">No Signal</p>
                 </div>
              )}
              
              {/* Playback Controls Overlay */}
              <div className="absolute bottom-4 left-1/2 -translate-x-1/2 flex items-center gap-4 bg-black/70 backdrop-blur-md px-4 py-2 rounded-full border border-white/10 z-20">
                 <button onClick={handleStop} className="text-white hover:text-red-400 transition-colors" title="Stop (Reset)">
                    <Square className="w-4 h-4 fill-current" />
                 </button>
                 
                 <button onClick={() => setIsPlaying(!isPlaying)} className="text-white hover:text-indigo-400 transition-colors scale-125">
                    {isPlaying ? <Pause className="w-5 h-5 fill-current" /> : <Play className="w-5 h-5 fill-current" />}
                 </button>

                 <div className="w-px h-4 bg-white/20 mx-2"></div>

                 {/* Playback Mode Toggle */}
                 <button 
                    onClick={() => setPlaybackMode(prev => prev === 'once' ? 'loop' : prev === 'loop' ? 'bounce' : 'once')}
                    className={`transition-colors ${playbackMode !== 'once' ? 'text-indigo-400' : 'text-neutral-500 hover:text-white'}`}
                    title={`Mode: ${playbackMode}`}
                 >
                    {playbackMode === 'once' && <ArrowRightLeft className="w-4 h-4 opacity-50" />}
                    {playbackMode === 'loop' && <Repeat className="w-4 h-4" />}
                    {playbackMode === 'bounce' && <Repeat1 className="w-4 h-4" />}
                 </button>

                 <span className="text-xs font-mono text-neutral-300 w-16 text-center">
                    {Math.floor(currentTime / 60)}:{Math.floor(currentTime % 60).toString().padStart(2, '0')}
                 </span>
              </div>
            </div>
            
            {status?.isGenerating && (
              <div className="mt-8 flex items-center gap-3 text-sm text-indigo-300 animate-pulse bg-indigo-900/20 px-4 py-2 rounded-full border border-indigo-500/20">
                <div className="w-2 h-2 bg-indigo-400 rounded-full animate-ping"></div>
                {status.progress}
              </div>
            )}
             {status?.error && (
              <div className="mt-8 flex items-center gap-3 text-sm text-red-300 bg-red-900/20 px-4 py-2 rounded-full border border-red-500/20">
                <AlertCircle className="w-4 h-4" />
                {status.error}
              </div>
            )}
          </div>
          
          <div className="p-4 bg-neutral-900 border-b border-neutral-800 flex items-center gap-4 h-16 shrink-0">
             <label className="flex items-center gap-2 px-4 py-2 bg-neutral-800 hover:bg-neutral-700 rounded text-sm text-white cursor-pointer transition-colors">
               <Upload className="w-4 h-4" /> Import Soundtrack
               <input type="file" accept="audio/*" className="hidden" onChange={handleSoundtrackUpload} />
             </label>
             <div className="h-6 w-px bg-neutral-800"></div>
             
             {/* Loop Controls */}
             <div className="flex items-center gap-2 bg-neutral-950 px-3 py-1 rounded border border-neutral-800">
                <span className="text-[10px] text-neutral-500 uppercase font-bold">Loop</span>
                <input 
                    type="number" 
                    value={loopStart} 
                    onChange={(e) => setLoopStart(Number(e.target.value))}
                    className="w-12 bg-neutral-800 border border-neutral-700 rounded text-xs px-1 text-center text-white"
                />
                <span className="text-neutral-600 text-xs">-</span>
                <input 
                    type="number" 
                    value={loopEnd} 
                    onChange={(e) => setLoopEnd(Number(e.target.value))}
                    className="w-12 bg-neutral-800 border border-neutral-700 rounded text-xs px-1 text-center text-white"
                />
             </div>

             <div className="h-6 w-px bg-neutral-800"></div>

             {selectedClip ? (
                <div className="flex items-center gap-4 animate-in fade-in slide-in-from-bottom-2">
                    <span className="text-xs text-indigo-300 font-bold uppercase truncate max-w-[150px]">{selectedClip.label}</span>
                    
                    {/* Duration Slider */}
                    <div className="flex items-center gap-3 bg-neutral-950 px-3 py-1 rounded-full border border-neutral-800">
                        <span className="text-xs text-neutral-500">Duration</span>
                        <input 
                            type="range" 
                            min="0.5" 
                            max="60" 
                            step="0.5"
                            value={selectedClip.duration}
                            onChange={(e) => updateClipDuration(selectedClip.id, parseFloat(e.target.value))}
                            className="w-24 h-1 bg-neutral-700 rounded-lg appearance-none cursor-pointer accent-indigo-500"
                        />
                        <span className="text-xs font-mono text-white w-8 text-right">{selectedClip.duration}s</span>
                    </div>

                    <button 
                       onClick={(e) => deleteClip(selectedClip.id, e)}
                       className="p-2 hover:bg-red-900/30 text-neutral-500 hover:text-red-400 rounded transition-colors"
                       title="Delete Clip"
                    >
                       <Trash2 className="w-4 h-4" />
                    </button>
                </div>
             ) : (
                <span className="text-xs text-neutral-500">Select a clip to edit</span>
             )}
          </div>
        </div>
      </div>

      {/* Sequencer */}
      <div className="h-72 bg-neutral-900 border-t border-neutral-800 flex flex-col select-none relative group shrink-0">
        {/* Timeline Header */}
        <div className="h-14 border-b border-neutral-800 flex items-center px-4 bg-neutral-900 text-xs font-mono text-neutral-500 sticky top-0 z-30">
          <div className="w-24 border-r border-neutral-800 h-full flex flex-col items-center justify-center flex-shrink-0 gap-1">
             <span className="text-[10px] font-bold">TIMELINE</span>
             <div className="flex items-center gap-1">
                <button onClick={() => handleZoom('out')} className="p-0.5 hover:text-white"><ZoomOut className="w-3 h-3" /></button>
                <button onClick={() => handleZoom('in')} className="p-0.5 hover:text-white"><ZoomIn className="w-3 h-3" /></button>
             </div>
          </div>
          <div 
            className="flex-1 relative h-full flex items-end pb-1 overflow-hidden cursor-pointer group/timeline" 
            ref={timelineRef} 
            onMouseDown={(e) => {
               const rect = e.currentTarget.getBoundingClientRect();
               const clickX = e.clientX - rect.left;
               setCurrentTime(clickX / zoom);
            }}
            onDoubleClick={handleAddMarker}
          >
             <div className="absolute top-1 left-2 text-[9px] text-neutral-600 opacity-0 group-hover/timeline:opacity-100 transition-opacity pointer-events-none">
                Double-click to add marker
             </div>

             {/* Scene Markers Track - NEW: VISUALIZE SCENES IN TIMELINE */}
             {sceneRegions.map((region) => (
                <div 
                    key={region.scene.id}
                    className="absolute top-5 h-4 border-l border-neutral-600 bg-neutral-800/50 hover:bg-indigo-900/30 text-[9px] text-neutral-400 px-1 truncate pointer-events-none"
                    style={{
                        left: `${region.start * zoom}px`,
                        width: `${(region.end - region.start) * zoom}px`
                    }}
                >
                    SC {region.scene.number}
                </div>
             ))}

             {/* Chapter Markers */}
             {chapterMarkers.map(marker => (
                 <div 
                    key={marker.id}
                    className="absolute top-0 group/marker cursor-pointer z-50 hover:z-50"
                    style={{ left: `${marker.time * zoom}px` }}
                    onClick={(e) => { e.stopPropagation(); /* allow selection logic here if needed */ }}
                 >
                    <div className="flex flex-col items-center -translate-x-1/2">
                        <Flag className="w-3 h-3 fill-amber-500 text-amber-500" />
                        <div className="h-full w-px bg-amber-500/50"></div>
                        <div className="absolute top-4 bg-black/80 text-amber-500 text-[9px] px-1 rounded whitespace-nowrap opacity-0 group-hover/marker:opacity-100 transition-opacity pointer-events-none">
                            {marker.label}
                        </div>
                        <button 
                            onClick={(e) => handleDeleteMarker(marker.id, e)}
                            className="absolute -top-3 right-0 opacity-0 group-hover/marker:opacity-100 text-red-500 hover:text-red-400 bg-black/50 rounded-full p-0.5"
                        >
                            <Trash2 className="w-2 h-2" />
                        </button>
                    </div>
                 </div>
             ))}

             {/* Loop Markers */}
             {(playbackMode === 'loop' || playbackMode === 'bounce') && (
                <>
                    <div className="absolute top-0 bottom-0 w-px bg-green-500/50 pointer-events-none" style={{ left: `${loopStart * zoom}px` }}>
                        <div className="bg-green-500 text-[8px] text-black px-1">START</div>
                    </div>
                    <div className="absolute top-0 bottom-0 w-px bg-red-500/50 pointer-events-none" style={{ left: `${loopEnd * zoom}px` }}>
                        <div className="bg-red-500 text-[8px] text-black px-1">END</div>
                    </div>
                    <div className="absolute top-0 h-1 bg-green-500/20 pointer-events-none" style={{ left: `${loopStart * zoom}px`, width: `${(loopEnd - loopStart) * zoom}px` }}></div>
                </>
             )}

             {Array.from({ length: Math.ceil(totalDuration / 5) + 1 }).map((_, i) => (
                <div key={i} className="absolute bottom-0 border-l border-neutral-700 h-2 pl-1 select-none pointer-events-none" style={{ left: `${i * 5 * zoom}px` }}>
                   {i * 5}s
                </div>
             ))}
             {/* Playhead */}
             <div 
                className="absolute top-0 bottom-0 w-px bg-red-500 z-40 pointer-events-none"
                style={{ left: `${currentTime * zoom}px` }}
             >
                <div className="w-3 h-3 -ml-1.5 bg-red-500 transform rotate-45 -mt-1.5"></div>
             </div>
          </div>
        </div>

        {/* Tracks Container */}
        <div className="flex-1 overflow-y-auto overflow-x-hidden relative custom-scrollbar">
          {['visual', 'audio', 'music'].map((trackType) => (
             <div 
                key={trackType} 
                className="h-24 border-b border-neutral-800 flex relative"
                onDragOver={handleTrackDragOver}
                onDrop={(e) => handleTrackDrop(e, trackType as TimelineItem['track'])}
             >
                <div className="w-24 bg-neutral-925 border-r border-neutral-800 flex flex-col items-center justify-center gap-1 z-10 flex-shrink-0">
                   {trackType === 'visual' && <Video className="w-4 h-4 text-blue-400" />}
                   {trackType === 'audio' && <Mic className="w-4 h-4 text-yellow-400" />}
                   {trackType === 'music' && <Music className="w-4 h-4 text-purple-400" />}
                   <span className="text-[10px] text-neutral-500 uppercase">{trackType}</span>
                </div>
                <div className="flex-1 bg-neutral-950 relative min-w-0" onClick={() => setSelectedClipId(null)}>
                   {localTimeline
                      .filter(t => t.track === trackType)
                      .map((item) => (
                         <div 
                            key={item.id}
                            onMouseDown={(e) => handleDragStart(e, item)}
                            onClick={(e) => { e.stopPropagation(); setSelectedClipId(item.id); }}
                            onMouseEnter={() => (trackType === 'audio' || trackType === 'music') && setHoveredAudioId(item.id)}
                            onMouseLeave={() => (trackType === 'audio' || trackType === 'music') && setHoveredAudioId(null)}
                            className={`absolute top-2 bottom-2 rounded overflow-hidden cursor-move group/item border hover:brightness-110 transition-all ${
                                selectedClipId === item.id 
                                ? 'border-white ring-2 ring-indigo-500/50 z-20' 
                                : item.track === 'visual' ? 'bg-blue-900/40 border-blue-700/50' 
                                : item.track === 'audio' ? 'bg-yellow-900/40 border-yellow-700/50' 
                                : 'bg-purple-900/40 border-purple-700/50'
                            }`}
                            style={{ 
                                left: `${item.startTime * zoom}px`, 
                                width: `${item.duration * zoom}px` 
                            }}
                         >
                             <div className={`px-2 py-1 text-[10px] truncate pointer-events-none select-none ${
                                 item.track === 'visual' ? 'text-blue-200' 
                                 : item.track === 'audio' ? 'text-yellow-200' 
                                 : 'text-purple-200'
                             }`}>
                                {item.label}
                             </div>
                             
                             {item.type === MediaType.VIDEO && (
                                <video src={item.src} className="w-full h-full object-cover opacity-30 pointer-events-none absolute top-0 left-0 -z-10" />
                             )}
                             {item.type === MediaType.IMAGE && (
                                <img src={item.src} className="w-full h-full object-cover opacity-50 pointer-events-none absolute top-0 left-0 -z-10" />
                             )}
                         </div>
                      ))}
                </div>
             </div>
          ))}
          
          {/* Audio Elements Container */}
          <div className="sr-only" aria-hidden="true">
            {localTimeline.filter(t => t.track === 'audio' || t.track === 'music').map(item => (
                <audio 
                    key={item.id} 
                    ref={el => {
                        if (el) audioRefs.current[item.id] = el;
                        else delete audioRefs.current[item.id];
                    }}
                    src={item.src} 
                    preload="auto"
                />
            ))}
          </div>

          {/* Vertical Playhead Line spanning all tracks */}
          <div 
              className="absolute top-0 bottom-0 w-px bg-red-500/50 z-0 pointer-events-none"
              style={{ left: `${(currentTime * zoom) + 96}px` }} // 96px is track header width
           ></div>
        </div>
      </div>
    </div>
  );
};

export default Studio;