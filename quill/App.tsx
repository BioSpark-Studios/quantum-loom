import React, { useState, useEffect } from 'react';
import { Project, DEMO_SCRIPT } from './types';
import ProjectList from './components/ProjectList';
import Studio from './components/Studio';
import MusicVideoStudio from './components/MusicVideoStudio';
import { Clapperboard, Music, Film } from 'lucide-react';
import { STAR_SHADOW_PROJECT } from './services/starShadowHeresy';

type AppMode = 'projects' | 'screenplay' | 'musicvideo';

const App: React.FC = () => {
  const [mode, setMode]                   = useState<AppMode>('projects');
  const [currentProject, setCurrentProject] = useState<Project | null>(null);
  const [projects, setProjects]           = useState<Project[]>([]);

  useEffect(() => {
    try {
      const saved = localStorage.getItem('quantum_quill_projects');
      let loadedProjects: Project[] = [];
      if (saved) {
        loadedProjects = JSON.parse(saved);
      }

      // Seed or Update example projects
      const existingStarShadowIndex = loadedProjects.findIndex(p => p.id === STAR_SHADOW_PROJECT.id);
      const isOutdated = existingStarShadowIndex !== -1 && 
        (!loadedProjects[existingStarShadowIndex].genesis.mythosContainers.some(c => c.name === 'BLUEPRINTS') ||
         !loadedProjects[existingStarShadowIndex].scriptContent.includes('DEFENDING THE THIRTEENTH SEED'));

      if (existingStarShadowIndex === -1) {
        // Doesn't exist, add it
        loadedProjects = [STAR_SHADOW_PROJECT, ...loadedProjects];
        localStorage.setItem('quantum_quill_projects', JSON.stringify(loadedProjects));
      } else if (isOutdated) {
        // Exists but old, update it
        loadedProjects[existingStarShadowIndex] = STAR_SHADOW_PROJECT;
        localStorage.setItem('quantum_quill_projects', JSON.stringify(loadedProjects));
        // If the current project is the one we just updated, refresh it
        if (currentProject?.id === STAR_SHADOW_PROJECT.id) {
          setCurrentProject(STAR_SHADOW_PROJECT);
        }
      }
      
      setProjects(loadedProjects);
    } catch (e) {
      console.error("Failed to load projects", e);
    }
  }, []);

  const saveProjects = (updatedProjects: Project[]) => {
    setProjects(updatedProjects);
    localStorage.setItem('quantum_quill_projects', JSON.stringify(updatedProjects));
  };

  const createProject = () => {
    const newProject: Project = {
      id: crypto.randomUUID(),
      name: `Untitled Script ${new Date().toLocaleDateString()}`,
      lastModified: Date.now(),
      scriptContent: DEMO_SCRIPT,
      scenes: [],
      timeline: [],
      characterVoices: {},
      chapterMarkers: [],
      genesis: {
        id: crypto.randomUUID(),
        name: "Project Genesis",
        description: "Core data container",
        mythosContainers: [],
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
        genre: ["Sci-Fi", "Fantasy"],
        themes: [],
        narrativeScope: "full_story" as any,
        setting: {
            location: "Unknown",
            timePeriod: "Epoch 1",
            atmosphere: "Neutral"
        },
        maxMythosContainers: 12,
        globalSettings: {},
        version: "2.6"
      }
    };
    const updated = [newProject, ...projects];
    saveProjects(updated);
    setCurrentProject(newProject);
  };

  const deleteProject = (id: string) => {
    const updated = projects.filter(p => p.id !== id);
    saveProjects(updated);
    if (currentProject?.id === id) setCurrentProject(null);
  };

  const updateProject = (project: Project) => {
    const updated = projects.map(p => p.id === project.id ? project : p);
    saveProjects(updated);
    setCurrentProject(project);
  };

  return (
    <div className="min-h-screen bg-neutral-950 text-neutral-200 flex flex-col">
      {/* Header */}
      <header className="h-14 border-b border-neutral-800 bg-neutral-900/50 backdrop-blur flex items-center px-6 justify-between z-10 shrink-0">
        <div className="flex items-center gap-3 cursor-pointer" onClick={() => { setMode('projects'); setCurrentProject(null); }}>
          <div className="w-7 h-7 bg-gradient-to-tr from-indigo-500 to-purple-600 rounded flex items-center justify-center">
            <Clapperboard className="w-4 h-4 text-white" />
          </div>
          <h1 className="font-display font-bold text-lg tracking-wider text-white">QUANTUM QUILL</h1>
        </div>

        {/* Mode tabs */}
        <div className="flex items-center gap-1 bg-neutral-900 border border-neutral-800 rounded-lg p-1">
          <button
            onClick={() => { setMode('projects'); setCurrentProject(null); }}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all"
            style={mode === 'projects' || mode === 'screenplay'
              ? { background: 'rgba(99,102,241,0.2)', color: '#818cf8' }
              : { color: '#666' }}
          >
            <Film className="w-3 h-3" /> Screenplay
          </button>
          <button
            onClick={() => setMode('musicvideo')}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all"
            style={mode === 'musicvideo'
              ? { background: 'rgba(220,60,120,0.2)', color: '#dc3c78' }
              : { color: '#666' }}
          >
            <Music className="w-3 h-3" /> Music Video
          </button>
        </div>

        {currentProject && mode === 'screenplay' && (
          <span className="text-xs text-indigo-300 font-medium">{currentProject.name}</span>
        )}
        {mode === 'musicvideo' && (
          <span className="text-xs font-medium" style={{ color: '#dc3c78' }}>Choreography · Sound · Behavior</span>
        )}
      </header>

      {/* Main Content */}
      <main className="flex-1 flex overflow-hidden">
        {mode === 'musicvideo' ? (
          <MusicVideoStudio />
        ) : currentProject ? (
          <Studio
            project={currentProject}
            onUpdate={updateProject}
            onBack={() => { setCurrentProject(null); setMode('projects'); }}
          />
        ) : (
          <ProjectList
            projects={projects}
            onCreate={createProject}
            onSelect={(p) => { setCurrentProject(p); setMode('screenplay'); }}
            onDelete={deleteProject}
            onImportLoom={(p) => { saveProjects([p, ...projects]); setCurrentProject(p); setMode('screenplay'); }}
          />
        )}
      </main>
    </div>
  );
};

export default App;