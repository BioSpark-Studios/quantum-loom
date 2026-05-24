import React from 'react';
import { Project } from '../types';
import { Plus, Trash2, FileText, Clock, Import } from 'lucide-react';
import { hasLoomData, readLoomWorld, importLoomScenes } from '../services/loomBridge';

interface ProjectListProps {
  projects: Project[];
  onCreate: () => void;
  onSelect: (project: Project) => void;
  onDelete: (id: string) => void;
  onImportLoom?: (project: Project) => void;
}

const ProjectList: React.FC<ProjectListProps> = ({ projects, onCreate, onSelect, onDelete, onImportLoom }) => {
  const loomAvailable = hasLoomData();

  const handleImportLoom = () => {
    if (!onImportLoom) return;
    const world = readLoomWorld();
    const scenes = importLoomScenes();
    const project: Project = {
      id: 'loom-import-' + Date.now(),
      name: world.projectName + ' — Quill',
      lastModified: Date.now(),
      scriptContent: scenes.map(s => `${s.number}. ${s.heading}\n${s.description}`).join('\n\n'),
      scenes,
      timeline: [],
      characterVoices: Object.fromEntries(world.characters.map(c => [c.name, 'Fenrir'])),
      chapterMarkers: [],
      genesis: { id: 'loom-g', name: world.projectName, mythosContainers: [], capsules: [], metadata: {} as any },
    } as any;
    onImportLoom(project);
  };

  return (
    <div className="w-full max-w-5xl mx-auto p-12">
      <div className="flex justify-between items-center mb-12">
        <div>
          <h2 className="text-3xl font-display font-bold text-white mb-2">Projects</h2>
          <p className="text-neutral-400">Manage your screenplays and generated timelines.</p>
        </div>
        <div className="flex gap-3">
          {loomAvailable && onImportLoom && (
            <button
              onClick={handleImportLoom}
              className="bg-amber-700/40 hover:bg-amber-600/50 border border-amber-600/30 text-amber-300 px-5 py-3 rounded-lg flex items-center gap-2 transition-all font-medium text-sm"
              title="Import scenes and characters from Quantum Loom"
            >
              <Import className="w-4 h-4" /> From Loom
            </button>
          )}
          <button
            onClick={onCreate}
            className="bg-indigo-600 hover:bg-indigo-500 text-white px-6 py-3 rounded-lg flex items-center gap-2 transition-all shadow-lg shadow-indigo-900/20 font-medium"
          >
            <Plus className="w-5 h-5" /> New Project
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {/* Create Card */}
        <div 
          onClick={onCreate}
          className="group border-2 border-dashed border-neutral-800 rounded-xl p-8 flex flex-col items-center justify-center gap-4 cursor-pointer hover:border-indigo-500/50 hover:bg-neutral-900/50 transition-all min-h-[200px]"
        >
          <div className="w-12 h-12 rounded-full bg-neutral-900 group-hover:bg-indigo-900/30 flex items-center justify-center transition-colors">
            <Plus className="w-6 h-6 text-neutral-500 group-hover:text-indigo-400" />
          </div>
          <span className="text-neutral-500 group-hover:text-indigo-300 font-medium">Create New Project</span>
        </div>

        {/* Project Cards */}
        {projects.map((project) => (
          <div 
            key={project.id}
            onClick={() => onSelect(project)}
            className="group bg-neutral-900 border border-neutral-800 rounded-xl p-6 cursor-pointer hover:border-indigo-500/50 hover:shadow-xl hover:shadow-indigo-900/10 transition-all relative overflow-hidden"
          >
            <div className="absolute top-0 right-0 p-4 opacity-0 group-hover:opacity-100 transition-opacity">
               <button 
                 onClick={(e) => { e.stopPropagation(); onDelete(project.id); }}
                 className="text-neutral-500 hover:text-red-400 p-2 hover:bg-red-900/20 rounded-lg transition-colors"
               >
                 <Trash2 className="w-4 h-4" />
               </button>
            </div>

            <div className="flex items-start justify-between mb-4">
              <div className="w-10 h-10 bg-indigo-900/30 rounded-lg flex items-center justify-center text-indigo-400">
                <FileText className="w-5 h-5" />
              </div>
            </div>

            <h3 className="text-xl font-bold text-white mb-2 group-hover:text-indigo-300 transition-colors truncate">
              {project.name}
            </h3>
            
            <p className="text-neutral-500 text-sm line-clamp-2 mb-6 font-script">
              {project.scriptContent.substring(0, 100)}...
            </p>

            <div className="flex items-center gap-4 text-xs text-neutral-500 border-t border-neutral-800 pt-4">
              <div className="flex items-center gap-1">
                <Clock className="w-3 h-3" />
                <span>{new Date(project.lastModified).toLocaleDateString()}</span>
              </div>
              <div>
                {project.scenes.length} Scenes
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default ProjectList;
