use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use serde::Deserialize;

// Define la estructura del JSON con los tipos convertidos
#[derive(Deserialize, Debug)]
struct JsonDataRaw {
    data: String,
    chunks: Vec<String>,
}

#[derive(Debug, Clone)]
struct JsonData {
    data: Vec<u8>,
    chunks: Vec<Vec<u8>>,
}

// Convierte el formato Raw a los valores numéricos
impl JsonData {
    fn from_raw(raw: JsonDataRaw) -> Result<Self, Box<dyn std::error::Error>> {
        let data = hex::decode(&raw.data)?;  // Convierte el campo 'data' a Vec<u8>
        let chunks = raw
            .chunks
            .into_iter()
            .map(|chunk| hex::decode(&chunk))
            .collect::<Result<Vec<Vec<u8>>, _>>()?;  // Convierte cada chunk a Vec<u8>
        
        Ok(JsonData { data, chunks })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use erasure_coding::*;

    // Función para leer y deserializar el JSON
    fn read_ec_test(filename: &str) -> Result<JsonData, Box<dyn std::error::Error>> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(filename);
        let mut file = File::open(&path).expect("Failed to open file");
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // Primero deserializamos al formato crudo (Raw)
        let raw_data: JsonDataRaw = serde_json::from_str(&content)?;
        
        // Convertimos a la estructura final con los valores numéricos
        let data = JsonData::from_raw(raw_data)?;

        Ok(data)
    }

    fn run_decode_chunks_test(test_data: JsonData) -> Vec<Segment> {
        let subshards: Vec<(u8, ChunkIndex, SubShard)> = test_data.chunks
            .iter()
            .enumerate()
            .map(|(chunk_idx, chunk)| {
                let mut subshard = [0u8; SUBSHARD_SIZE];  
                subshard[..chunk.len()].copy_from_slice(chunk);  
                (1u8, ChunkIndex(chunk_idx as u16), subshard)  
            })
            .collect();
    
        let mut subshards_iter = subshards
            .iter()
            .map(|(a, b, c)| (*a, *b, c));
    
        let mut decoder = SubShardDecoder::new().unwrap();
        let (reconstructed_segments, _nb_decode) = decoder.reconstruct(&mut subshards_iter).unwrap();
    
        reconstructed_segments.into_iter().map(|(_, segment)| segment).collect()
    }
    
    fn run_encode_data_test(test_data: JsonData) -> Vec<Box<[SubShard; TOTAL_SHARDS]>> {

        let mut segment_data = [0u8; SEGMENT_SIZE];  
        let data_len = test_data.data.len();  
        segment_data[..data_len].copy_from_slice(&test_data.data[..data_len]);

        let segment = Segment {
            data: Box::new(segment_data),  
            index: 0,  
        };

        let mut encoder = SubShardEncoder::new().unwrap();  
        encoder.construct_chunks(&[segment]).unwrap()
    }

    #[test]
    fn test_ec_x() {

        // Decode chunks test
        let test_data = read_ec_test("tests/test_vectors/jamtestvectors/erasure_coding/vectors/ec_x.json").unwrap();
        let segments = run_decode_chunks_test(test_data.clone());

        if let Some(first_segment) = segments.get(0) {
            assert_eq!(&first_segment.data[..342], &test_data.data[..342]);
        } else {
            panic!("ec_x.json No segments found!");
        }
      
        // Encode data test
        let chunks_encoded = run_encode_data_test(test_data.clone());

        for (chunk_idx, chunk) in test_data.chunks.iter().enumerate() {
            assert_eq!(&chunks_encoded[0][chunk_idx][..2], &chunk[..2]);
        }
    }

    #[test]
    fn test_ec_1() {

        // Decode chunks test
        let test_data = read_ec_test("tests/test_vectors/jamtestvectors/erasure_coding/vectors/ec_1.json").unwrap();
        let segments = run_decode_chunks_test(test_data.clone());

        if let Some(first_segment) = segments.get(0) {
            assert_eq!(&first_segment.data[..1], &test_data.data[..1]);
        } else {
            panic!("ec_x.json No segments found!");
        }
      
        // Encode data test
        let chunks_encoded = run_encode_data_test(test_data.clone());
        
        for (chunk_idx, chunk) in test_data.chunks.iter().enumerate() {
            assert_eq!(&chunks_encoded[0][chunk_idx][..2], &chunk[..2]);
        }
    }

}

