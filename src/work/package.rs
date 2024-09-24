use crate::refine::RefineContext;
use crate::codec::*;

#[derive(Default, Clone)]
pub struct ImportSpec {
    pub tree_root: [u8; 32],
    pub index: u16,
}

#[derive(Default, Clone)]
pub struct ExtrinsicSpec {
    pub hash: [u8; 32],
    pub len: u32,
}

struct Authorizer {
    code_hash: [u8; 32],
    params: Vec<u8>,
}

enum WorkExecResult {
    ok = 0,
    out_of_gas = 1,
    panic = 2,
    bad_code = 3,
    code_oversize = 4,
}

pub struct WorkItem {
    service: u32,
    code_hash: [u8; 32],
    payload: Vec<u8>,
    gas_limit: u64,
    import_segments: Vec<ImportSpec>,
    extrinsic: Vec<ExtrinsicSpec>,
    result: u8,
}

pub struct WorkPackage {
    authorization: Vec<u8>,
    auth_code_host: u32,
    authorizer: Authorizer,
    context: RefineContext,
    items: [WorkItem; 4],
}

pub fn decode_work_pkg(work_pkg_blob: &Vec<u8>) -> WorkPackage {

    let authorization_usize: Vec<usize> = decode_variable_length(work_pkg_blob);
    let authorization: Vec<u8> = authorization_usize.iter().map(|&x| x as u8).collect();
    let mut index = authorization.len();
    let auth_code_host: u32 = decode_trivial(&work_pkg_blob[index..index + 4].to_vec()) as u32;
    index += 4;
    let mut code_hash = [0u8; 32];
    code_hash.copy_from_slice(&work_pkg_blob[index..index + 32]);
    index += 32;
    let params_usize: Vec<usize> = decode_variable_length(&work_pkg_blob[index..].to_vec());
    let params: Vec<u8> = params_usize.iter().map(|&x| x as u8).collect();
    index += params.len();
    let authorizer: Authorizer {code_hash, params};
    let context: RefineContext = decode_refine_ctx(&work_pkg_blob[index..].to_vec());
    index += 164;
    let items: [WorkItem; 4];
    for i in 0..4 {
        items[i] = decode_work_item(&work_pkg_blob)
    }
}

pub fn encode_work_pkg(work_pkg: &WorkPackage) -> Vec<u8> {

    vec![]
}


pub fn decode_work_item(work_item_blob: &Vec<u8>) -> WorkItem {

    let service: u32 = decode_trivial(&work_item_blob[0..4].to_vec()) as u32;
    let mut code_hash = [0u8; 32];
    code_hash.copy_from_slice(&work_item_blob[4..36]);
    let payload_usize: Vec<usize> = decode_variable_length(&work_item_blob[36..].to_vec());
    let payload: Vec<u8> = payload_usize.iter().map(|&x| x as u8).collect();
    let mut index: usize = 1 + 36 + payload.len();
    let gas_limit: u64 = decode_trivial(&work_item_blob[index..index + 8].to_vec()) as u64;
    index += 8;
    let num_segments = work_item_blob[index] as usize;
    index += 1;
    let mut import_segments: Vec<ImportSpec> = vec![ImportSpec::default(); num_segments];
    for i in 0..num_segments {
        import_segments[i as usize].tree_root.copy_from_slice(&work_item_blob[index..index + 32]);
        index += 32;
        import_segments[i as usize].index = decode_trivial(&work_item_blob[index..index + 2].to_vec()) as u16;
        index += 2;
    }
    let num_extrinsics = work_item_blob[index] as usize;
    index += 1;
    let mut extrinsic: Vec<ExtrinsicSpec> = vec![ExtrinsicSpec::default(); num_extrinsics];
    for i in 0..num_extrinsics {
        extrinsic[i as usize].hash.copy_from_slice(&work_item_blob[index..index + 32]);
        index += 32;
        extrinsic[i as usize].len = decode_trivial(&work_item_blob[index..index + 4].to_vec()) as u32;
        index += 4;
    }
    let result = work_item_blob[index];

    /*println!("service: {}", service);
    println!("code_hash: {:02x?}", code_hash);
    println!("payload: {:02x?}", payload);
    println!("gas_limit: {:02x?}", gas_limit);
    println!("import_segments tree_root: {:02x?}", import_segments[0].tree_root);
    println!("import_segments index: {:02x?}", import_segments[0].index);
    println!("extrinsic hash: {:?}", extrinsic[0].hash);
    println!("extrinsic len: {:?}", extrinsic[0].len);
    println!("result: {:?}", result);*/

    WorkItem {
        service,
        code_hash,
        payload,
        gas_limit,
        import_segments,
        extrinsic,
        result,
    }
}

pub fn encode_work_item(work_item: &WorkItem) -> Vec<u8> {

    let mut work_item_blob: Vec<u8> = vec![];

    work_item_blob.extend_from_slice(&encode_trivial(work_item.service as usize, 4));
    work_item_blob.extend_from_slice(&work_item.code_hash);
    let payload_usize: Vec<usize> = work_item.payload.iter().map(|&x| x as usize).collect();
    work_item_blob.extend_from_slice(&encode_variable_length(&payload_usize));
    work_item_blob.extend_from_slice(&encode_trivial(work_item.gas_limit as usize, 8));
    work_item_blob.push(work_item.import_segments.len() as u8);
    for i in 0..work_item.import_segments.len() {
        work_item_blob.extend_from_slice(&work_item.import_segments[i as usize].tree_root);
        work_item_blob.extend_from_slice(&encode_trivial(work_item.import_segments[i as usize].index as usize, 2));
    }
    work_item_blob.push(work_item.extrinsic.len() as u8);
    for i in 0..work_item.extrinsic.len() {
        work_item_blob.extend_from_slice(&work_item.extrinsic[i as usize].hash);
        work_item_blob.extend_from_slice(&encode_trivial(work_item.extrinsic[i as usize].len as usize, 4));
    }
    work_item_blob.extend_from_slice(&encode_trivial(work_item.result as usize, 2));

    return work_item_blob;
}
