check:
	cargo clippy --examples --workspace --all-targets --all-features
run:
	# cargo run --example scene_viewer --release -- .idea/bistro/Bistro_v5_2/BistroExterior.fbx
	cargo run --example cube 