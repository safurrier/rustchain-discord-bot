use crate::config::LLM_MODEL;
use llm_chain::options::ModelRef;
use llm_chain::options::Opt;
use llm_chain::options::Options;

// Make the options function public
pub fn options() -> Options {
    let mut builder = Options::builder();
    builder.add_option(Opt::Model(ModelRef::from_model_name(LLM_MODEL)));
    // Add any other LLM options here
    // https://github.com/sobelio/llm-chain/blob/d1c2abfecefa352cac2604d2a7aa2a84f074f641/crates/llm-chain/src/options.rs#L331
    // options::Opt
    builder.build()
}
