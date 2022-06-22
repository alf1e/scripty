use std::path::Path;

pub fn init_stt() {
    let cfg = scripty_config::get_config();

    crate::models::load_models(Path::new(&cfg.model_dir));

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl-c");

        info!("deallocating models");
        crate::models::deallocate_models();
    });
}
