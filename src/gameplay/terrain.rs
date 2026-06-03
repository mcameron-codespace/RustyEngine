use std::collections::HashMap;

/// Represents a loaded coordinate chunk key in the world grid (X, Z).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub z: i32,
}

/// Vertices and Indices data ready to be pushed to GPU buffers
#[derive(Debug, Clone)]
pub struct ChunkMesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// Handles generation of survival terrain heightmaps and static assets (foliage markers).
pub struct TerrainGenerator {
    pub chunk_size: f32,
    pub resolution: usize,
}

impl TerrainGenerator {
    pub fn new(chunk_size: f32, resolution: usize) -> Self {
        Self {
            chunk_size,
            resolution,
        }
    }

    /// Generate heightmap vertices and indices for a dynamic chunk.
    /// This is designed to be offloaded to worker thread-pools to prevent stalling the main loop.
    pub fn generate_chunk_mesh(&self, coords: ChunkCoords) -> ChunkMesh {
        let mut vertices = Vec::with_capacity((self.resolution + 1) * (self.resolution + 1));
        let mut indices = Vec::with_capacity(self.resolution * self.resolution * 6);

        let step = self.chunk_size / self.resolution as f32;
        let start_x = coords.x as f32 * self.chunk_size;
        let start_z = coords.z as f32 * self.chunk_size;

        // 1. Compute vertex positions using a procedural height equation (mock noise)
        for z in 0..=self.resolution {
            let world_z = start_z + (z as f32 * step);
            for x in 0..=self.resolution {
                let world_x = start_x + (x as f32 * step);
                
                // Simple sine-wave landscape approximation
                let height = (world_x * 0.1).sin() * (world_z * 0.1).cos() * 2.5;
                
                vertices.push([world_x, height, world_z]);
            }
        }

        // 2. Generate triangle strips matching vertex grids
        let row_stride = self.resolution + 1;
        for z in 0..self.resolution {
            for x in 0..self.resolution {
                let current = z * row_stride + x;
                let next = current + 1;
                let bottom = (z + 1) * row_stride + x;
                let bottom_next = bottom + 1;

                // Triangle 1
                indices.push(current as u32);
                indices.push(bottom as u32);
                indices.push(next as u32);

                // Triangle 2
                indices.push(next as u32);
                indices.push(bottom as u32);
                indices.push(bottom_next as u32);
            }
        }

        ChunkMesh { vertices, indices }
    }
}

/// Component managing active streaming chunks around a player entity.
pub struct TerrainStreamingManager {
    pub loaded_chunks: HashMap<ChunkCoords, ChunkMesh>,
    pub view_distance_chunks: i32,
}

impl TerrainStreamingManager {
    pub fn new(view_distance_chunks: i32) -> Self {
        Self {
            loaded_chunks: HashMap::new(),
            view_distance_chunks,
        }
    }

    /// Update dynamic streaming based on the player's world position.
    pub fn update_streaming(
        &mut self, 
        player_pos: [f32; 3], 
        generator: &TerrainGenerator,
        mut upload_to_gpu: impl FnMut(ChunkCoords, &ChunkMesh),
    ) {
        let chunk_size = generator.chunk_size;
        let center_x = (player_pos[0] / chunk_size).floor() as i32;
        let center_z = (player_pos[2] / chunk_size).floor() as i32;

        let mut chunks_to_keep = HashMap::new();

        // Load chunks within radius
        for dz in -self.view_distance_chunks..=self.view_distance_chunks {
            for dx in -self.view_distance_chunks..=self.view_distance_chunks {
                let coords = ChunkCoords { x: center_x + dx, z: center_z + dz };

                if let Some(mesh) = self.loaded_chunks.remove(&coords) {
                    chunks_to_keep.insert(coords, mesh);
                } else {
                    // Generate new chunk
                    let mesh = generator.generate_chunk_mesh(coords);
                    upload_to_gpu(coords, &mesh);
                    chunks_to_keep.insert(coords, mesh);
                }
            }
        }

        // Drop out-of-range chunks (rest of loaded_chunks deallocates cleanly)
        self.loaded_chunks = chunks_to_keep;
    }
}