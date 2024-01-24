use minitrace::collector::TraceId;

pub fn generate_trace_id(tenant_id: u32) -> TraceId {
  let uniq: u32 = rand::random();

  TraceId(((tenant_id as u128) << 32) | (uniq as u128))
}

pub fn extract_tenant_id(trace_id: TraceId) -> u32 {
  (trace_id.0 >> 32) as u32
}
