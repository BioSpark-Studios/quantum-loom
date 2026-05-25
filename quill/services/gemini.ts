import { GoogleGenAI, Type, Modality } from "@google/genai";
import { Scene, DialogueLine, GenesisContainer, MythosContainer, BaseCapsule, NarrativeScope } from "../types";
import { readLoomWorld, worldContextPrompt, hasLoomData } from "./loomBridge";

// Helper to ensure we have a key for Veo, which requires paid tier
export const ensureApiKey = async (): Promise<void> => {
  const aiStudio = (window as any).aistudio;
  if (aiStudio && aiStudio.hasSelectedApiKey) {
    const hasKey = await aiStudio.hasSelectedApiKey();
    if (!hasKey && aiStudio.openSelectKey) {
      await aiStudio.openSelectKey();
    }
  }
};

const getAI = () => new GoogleGenAI({ apiKey: process.env.API_KEY });

/**
 * Analyzes the raw screenplay text and breaks it into structured JSON.
 * Now acts as the "Lore Taxonomist" creating Genesis v2.6 structures.
 */
export const analyzeScript = async (scriptText: string): Promise<{ scenes: Scene[], genesis: GenesisContainer }> => {
  const ai = getAI();

  const loomCtx = hasLoomData() ? `\n\nQuantum Loom world context (cross-reference entities):\n${worldContextPrompt(readLoomWorld())}` : '';

  const response = await ai.models.generateContent({
    model: 'gemini-2.0-flash',
    contents: `Act as the "Lore Taxonomist" for the Quantum Quill Core. Analyze the following screenplay text and extract data into the Genesis Container (v2.6) format.${loomCtx}

    1. Break the script down into SCENES.
    2. Extract "Core Data Assets" (Capsules) from the script into MYTHOS CONTAINERS.
       - If Loom world context is provided above, align entity names with known characters, factions, and locations.
       - Otherwise identify characters, factions, locations, artifacts, and rules from the text itself.

    Organize Capsules into these Mythos Types:
    - "Social Stratification" (Factions/Races)
    - "Spatial Topography" (Locations/Realms)
    - "Cosmological Constants" (Rules/Physics)
    - "Narrative Drivers" (Characters)
    
    Return a JSON object with two keys: "scenes" (array) and "genesis_data" (object).
    
    Screenplay:
    ${scriptText}`,
    config: {
      responseMimeType: "application/json",
      responseSchema: {
        type: Type.OBJECT,
        properties: {
          scenes: {
            type: Type.ARRAY,
            items: {
              type: Type.OBJECT,
              properties: {
                number: { type: Type.INTEGER },
                heading: { type: Type.STRING },
                description: { type: Type.STRING },
                characters: { type: Type.ARRAY, items: { type: Type.STRING } },
                dialogue: {
                  type: Type.ARRAY,
                  items: {
                    type: Type.OBJECT,
                    properties: {
                      character: { type: Type.STRING },
                      text: { type: Type.STRING },
                      parenthetical: { type: Type.STRING }
                    }
                  }
                }
              }
            }
          },
          genesis_data: {
            type: Type.OBJECT,
            properties: {
              name: { type: Type.STRING },
              description: { type: Type.STRING },
              themes: { type: Type.ARRAY, items: { type: Type.STRING } },
              mythos: {
                type: Type.ARRAY,
                items: {
                  type: Type.OBJECT,
                  properties: {
                    name: { type: Type.STRING },
                    description: { type: Type.STRING },
                    mythosType: { type: Type.STRING },
                    icon: { type: Type.STRING },
                    color: { type: Type.STRING },
                    capsules: {
                      type: Type.ARRAY,
                      items: {
                        type: Type.OBJECT,
                        properties: {
                          name: { type: Type.STRING },
                          description: { type: Type.STRING },
                          kind: { type: Type.STRING },
                          tags: { type: Type.ARRAY, items: { type: Type.STRING } },
                          properties: { type: Type.OBJECT, properties: { atmosphere: { type: Type.STRING } } }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  });

  if (response.text) {
    const data = JSON.parse(response.text);
    
    // Process Scenes
    const scenes = data.scenes.map((s: any) => ({
      ...s,
      id: crypto.randomUUID(),
      dialogue: s.dialogue?.map((d: any) => ({ ...d, id: crypto.randomUUID() })) || []
    }));

    // Process Genesis Data
    const mythosContainers: MythosContainer[] = data.genesis_data.mythos.map((m: any) => ({
      id: crypto.randomUUID(),
      name: m.name,
      description: m.description,
      mythosType: m.mythosType,
      color: m.color || "#4A90E2",
      icon: m.icon || "📦",
      maxCapsules: 12,
      properties: {},
      created: new Date().toISOString(),
      modified: new Date().toISOString(),
      tags: [],
      capsules: m.capsules.map((c: any) => ({
        id: crypto.randomUUID(),
        name: c.name,
        description: c.description,
        kind: c.kind,
        glyphs: [],
        traits: [],
        tags: c.tags || [],
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
        author: "AI Taxonomist",
        version: "1.0",
        properties: c.properties || {}
      }))
    }));

    const genesis: GenesisContainer = {
      id: crypto.randomUUID(),
      name: data.genesis_data.name || "Xyrona Prime Genesis",
      description: data.genesis_data.description || "Core Data extracted from screenplay",
      mythosContainers,
      genre: ["Sci-Fantasy", "Verticality"],
      themes: data.genesis_data.themes || [],
      narrativeScope: NarrativeScope.FULL_STORY,
      setting: {
        location: "Xyrona Prime",
        timePeriod: "Epoch 4",
        atmosphere: "Resonant"
      },
      maxMythosContainers: 12,
      globalSettings: {},
      version: "2.6",
      created: new Date().toISOString(),
      modified: new Date().toISOString()
    };

    return { scenes, genesis };
  }
  
  throw new Error("Failed to parse script");
};

/**
 * Generates a video using Veo based on a scene description.
 */
export const generateSceneVideo = async (prompt: string): Promise<string> => {
  await ensureApiKey(); // Veo requires paid key
  const ai = getAI();

  // Veo operation
  let operation = await ai.models.generateVideos({
    model: 'veo-3.1-fast-generate-preview',
    prompt: `Cinematic shot, photorealistic, 4k. ${prompt}`,
    config: {
      numberOfVideos: 1,
      resolution: '1080p',
      aspectRatio: '16:9'
    }
  });

  // Polling loop
  while (!operation.done) {
    await new Promise(resolve => setTimeout(resolve, 5000));
    operation = await ai.operations.getVideosOperation({ operation: operation });
  }

  const uri = operation.response?.generatedVideos?.[0]?.video?.uri;
  if (!uri) throw new Error("No video URI returned");

  // Fetch with API key to get the actual blob
  const videoRes = await fetch(`${uri}&key=${process.env.API_KEY}`);
  const blob = await videoRes.blob();
  return URL.createObjectURL(blob);
};

/**
 * Generates an image using Imagen based on a scene description.
 */
export const generateSceneImage = async (prompt: string): Promise<string> => {
  const ai = getAI();

  const response = await ai.models.generateImages({
    model: 'imagen-4.0-generate-001',
    prompt: `Cinematic movie still, photorealistic, high detail. ${prompt}`,
    config: {
      numberOfImages: 1,
      outputMimeType: 'image/jpeg',
      aspectRatio: '16:9',
    },
  });

  const base64Image = response.generatedImages?.[0]?.image?.imageBytes;
  if (!base64Image) throw new Error("No image generated");

  // Create a blob URL
  const binaryString = atob(base64Image);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  const blob = new Blob([bytes], { type: 'image/jpeg' });
  return URL.createObjectURL(blob);
};

/**
 * Generates speech for a dialogue line.
 */
export const generateDialogueAudio = async (text: string, voiceName: string = 'Fenrir'): Promise<string> => {
  const ai = getAI();

  const response = await ai.models.generateContent({
    model: "gemini-2.5-flash-preview-tts",
    contents: {
      parts: [{ text: text }]
    },
    config: {
      responseModalities: [Modality.AUDIO],
      speechConfig: {
        voiceConfig: {
          prebuiltVoiceConfig: { voiceName }
        }
      }
    }
  });

  const base64Audio = response.candidates?.[0]?.content?.parts?.find(p => p.inlineData)?.inlineData?.data;
  if (!base64Audio) throw new Error("No audio generated");

  // Convert base64 to Blob
  const binaryString = atob(base64Audio);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  const blob = new Blob([bytes], { type: 'audio/pcm;rate=24000' }); 
  const wavBlob = pcmToWav(bytes, 24000);
  return URL.createObjectURL(wavBlob);
};

// Helper to add WAV header so browser <audio> can play it
function pcmToWav(pcmData: Uint8Array, sampleRate: number): Blob {
  const numChannels = 1;
  const bitsPerSample = 16;
  const byteRate = (sampleRate * numChannels * bitsPerSample) / 8;
  const blockAlign = (numChannels * bitsPerSample) / 8;
  const dataSize = pcmData.length;
  const buffer = new ArrayBuffer(44 + dataSize);
  const view = new DataView(buffer);

  // RIFF identifier
  writeString(view, 0, 'RIFF');
  // file length
  view.setUint32(4, 36 + dataSize, true);
  // RIFF type
  writeString(view, 8, 'WAVE');
  // format chunk identifier
  writeString(view, 12, 'fmt ');
  // format chunk length
  view.setUint32(16, 16, true);
  // sample format (raw)
  view.setUint16(20, 1, true);
  // channel count
  view.setUint16(22, numChannels, true);
  // sample rate
  view.setUint32(24, sampleRate, true);
  // byte rate (sample rate * block align)
  view.setUint32(28, byteRate, true);
  // block align (channel count * bytes per sample)
  view.setUint16(32, blockAlign, true);
  // bits per sample
  view.setUint16(34, bitsPerSample, true);
  // data chunk identifier
  writeString(view, 36, 'data');
  // data chunk length
  view.setUint32(40, dataSize, true);

  // write the PCM samples
  const dataView = new Uint8Array(buffer, 44);
  dataView.set(pcmData);

  return new Blob([buffer], { type: 'audio/wav' });
}

function writeString(view: DataView, offset: number, string: string) {
  for (let i = 0; i < string.length; i++) {
    view.setUint8(offset + i, string.charCodeAt(i));
  }
}