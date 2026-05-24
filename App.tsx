import React, { useState, useEffect } from 'react';
import { Project, DEMO_SCRIPT } from './types';
import ProjectList from './components/ProjectList';
import Studio from './components/Studio';
import { Layout, Clapperboard } from 'lucide-react';
import { STAR_SHADOW_PROJECT } from './services/starShadowHeresy';

const App: React.FC = () => {
  const [currentProject, setCurrentProject] = useState<Project | null>(null);
  const [projects, setProjects] = useState<Project[]>([]);

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
      <header className="h-16 border-b border-neutral-800 bg-neutral-900/50 backdrop-blur flex items-center px-6 justify-between z-10">
        <div className="flex items-center gap-3 cursor-pointer" onClick={() => setCurrentProject(null)}>
          <div className="w-8 h-8 bg-gradient-to-tr from-indigo-500 to-purple-600 rounded flex items-center justify-center">
             <Clapperboard className="w-5 h-5 text-white" />
          </div>
          <h1 className="font-display font-bold text-xl tracking-wider text-white">QUANTUM QUILL</h1>
        </div>
        
        {currentProject && (
          <div className="flex items-center gap-4 text-sm">
             <span className="text-neutral-400">Editing:</span>
             <span className="font-semibold text-indigo-300">{currentProject.name}</span>
          </div>
        )}
      </header>

      {/* Main Content */}
      <main className="flex-1 flex overflow-hidden">
        {currentProject ? (
          <Studio 
            project={currentProject} 
            onUpdate={updateProject} 
            onBack={() => setCurrentProject(null)}
          />
        ) : (
          <ProjectList 
            projects={projects} 
            onCreate={createProject} 
            onSelect={setCurrentProject}
            onDelete={deleteProject}
          />
        )}
      </main>
    </div>
  );
};

export default App;