use aw_core::{AWPacket, PacketType};

use crate::{
    AwInstance, PkwareDcl, SdkError, SdkResult,
    msg::handler::from_world::attributes::WorldAttributes,
};

pub fn world_attribute_change(
    instance: &mut AwInstance,
    attributes: &WorldAttributes,
) -> SdkResult<()> {
    let mut packet = AWPacket::new(PacketType::AttributeChange);

    if let Some(allow_3_axis_rotation) = &attributes.allow_3_axis_rotation {
        packet.add_string(0u16, allow_3_axis_rotation.clone());
    }
    if let Some(allow_avatar_collision) = &attributes.allow_avatar_collision {
        packet.add_string(1u16, allow_avatar_collision.clone());
    }
    if let Some(allow_citizen_whisper) = &attributes.allow_citizen_whisper {
        packet.add_string(2u16, allow_citizen_whisper.clone());
    }
    if let Some(allow_flying) = &attributes.allow_flying {
        packet.add_string(3u16, allow_flying.clone());
    }
    if let Some(allow_object_select) = &attributes.allow_object_select {
        packet.add_string(4u16, allow_object_select.clone());
    }
    if let Some(allow_passthru) = &attributes.allow_passthru {
        packet.add_string(5u16, allow_passthru.clone());
    }
    if let Some(allow_teleport) = &attributes.allow_teleport {
        packet.add_string(6u16, allow_teleport.clone());
    }
    if let Some(allow_tourist_build) = &attributes.allow_tourist_build {
        packet.add_string(7u16, allow_tourist_build.clone());
    }
    if let Some(allow_tourist_whisper) = &attributes.allow_tourist_whisper {
        packet.add_string(8u16, allow_tourist_whisper.clone());
    }
    if let Some(always_show_names) = &attributes.always_show_names {
        packet.add_string(9u16, always_show_names.clone());
    }
    if let Some(ambient_light_blue) = &attributes.ambient_light_blue {
        packet.add_string(10u16, ambient_light_blue.clone());
    }
    if let Some(ambient_light_green) = &attributes.ambient_light_green {
        packet.add_string(11u16, ambient_light_green.clone());
    }
    if let Some(ambient_light_red) = &attributes.ambient_light_red {
        packet.add_string(12u16, ambient_light_red.clone());
    }
    if let Some(avatar_refresh_rate) = &attributes.avatar_refresh_rate {
        packet.add_string(13u16, avatar_refresh_rate.clone());
    }
    if let Some(backdrop) = &attributes.backdrop {
        packet.add_string(14u16, backdrop.clone());
    }
    if let Some(bots_right) = &attributes.bots_right {
        packet.add_string(15u16, bots_right.clone());
    }
    if let Some(build_number) = &attributes.build_number {
        packet.add_string(16u16, build_number.clone());
    }
    if let Some(build_right) = &attributes.build_right {
        packet.add_string(17u16, build_right.clone());
    }
    if let Some(buoyancy) = &attributes.buoyancy {
        packet.add_string(18u16, buoyancy.clone());
    }
    if let Some(cell_limit) = &attributes.cell_limit {
        packet.add_string(19u16, cell_limit.clone());
    }
    if let Some(clouds_layer1_mask) = &attributes.clouds_layer1_mask {
        packet.add_string(20u16, clouds_layer1_mask.clone());
    }
    if let Some(clouds_layer1_opacity) = &attributes.clouds_layer1_opacity {
        packet.add_string(21u16, clouds_layer1_opacity.clone());
    }
    if let Some(clouds_layer1_speed_x) = &attributes.clouds_layer1_speed_x {
        packet.add_string(22u16, clouds_layer1_speed_x.clone());
    }
    if let Some(clouds_layer1_speed_z) = &attributes.clouds_layer1_speed_z {
        packet.add_string(23u16, clouds_layer1_speed_z.clone());
    }
    if let Some(clouds_layer1_texture) = &attributes.clouds_layer1_texture {
        packet.add_string(24u16, clouds_layer1_texture.clone());
    }
    if let Some(clouds_layer1_tile) = &attributes.clouds_layer1_tile {
        packet.add_string(25u16, clouds_layer1_tile.clone());
    }
    if let Some(clouds_layer2_mask) = &attributes.clouds_layer2_mask {
        packet.add_string(26u16, clouds_layer2_mask.clone());
    }
    if let Some(clouds_layer2_opacity) = &attributes.clouds_layer2_opacity {
        packet.add_string(27u16, clouds_layer2_opacity.clone());
    }
    if let Some(clouds_layer2_speed_x) = &attributes.clouds_layer2_speed_x {
        packet.add_string(28u16, clouds_layer2_speed_x.clone());
    }
    if let Some(clouds_layer2_speed_z) = &attributes.clouds_layer2_speed_z {
        packet.add_string(29u16, clouds_layer2_speed_z.clone());
    }
    if let Some(clouds_layer2_texture) = &attributes.clouds_layer2_texture {
        packet.add_string(30u16, clouds_layer2_texture.clone());
    }
    if let Some(clouds_layer2_tile) = &attributes.clouds_layer2_tile {
        packet.add_string(31u16, clouds_layer2_tile.clone());
    }
    if let Some(clouds_layer3_mask) = &attributes.clouds_layer3_mask {
        packet.add_string(32u16, clouds_layer3_mask.clone());
    }
    if let Some(clouds_layer3_opacity) = &attributes.clouds_layer3_opacity {
        packet.add_string(33u16, clouds_layer3_opacity.clone());
    }
    if let Some(clouds_layer3_speed_x) = &attributes.clouds_layer3_speed_x {
        packet.add_string(34u16, clouds_layer3_speed_x.clone());
    }
    if let Some(clouds_layer3_speed_z) = &attributes.clouds_layer3_speed_z {
        packet.add_string(35u16, clouds_layer3_speed_z.clone());
    }
    if let Some(clouds_layer3_texture) = &attributes.clouds_layer3_texture {
        packet.add_string(36u16, clouds_layer3_texture.clone());
    }
    if let Some(clouds_layer3_tile) = &attributes.clouds_layer3_tile {
        packet.add_string(37u16, clouds_layer3_tile.clone());
    }
    if let Some(creation_timestamp) = &attributes.creation_timestamp {
        packet.add_string(38u16, creation_timestamp.clone());
    }
    if let Some(disable_avatar_list) = &attributes.disable_avatar_list {
        packet.add_string(39u16, disable_avatar_list.clone());
    }
    if let Some(disable_chat) = &attributes.disable_chat {
        packet.add_string(40u16, disable_chat.clone());
    }
    if let Some(disable_create_url) = &attributes.disable_create_url {
        packet.add_string(41u16, disable_create_url.clone());
    }
    if let Some(eject_right) = &attributes.eject_right {
        packet.add_string(42u16, eject_right.clone());
    }
    if let Some(eminent_domain_right) = &attributes.eminent_domain_right {
        packet.add_string(43u16, eminent_domain_right.clone());
    }
    if let Some(enable_referer) = &attributes.enable_referer {
        packet.add_string(44u16, enable_referer.clone());
    }
    if let Some(enable_terrain) = &attributes.enable_terrain {
        packet.add_string(45u16, enable_terrain.clone());
    }
    if let Some(enter_right) = &attributes.enter_right {
        packet.add_string(46u16, enter_right.clone());
    }
    if let Some(entry_point) = &attributes.entry_point {
        packet.add_string(47u16, entry_point.clone());
    }
    if let Some(expiration) = &attributes.expiration {
        packet.add_string(48u16, expiration.clone());
    }
    if let Some(fog_blue) = &attributes.fog_blue {
        packet.add_string(49u16, fog_blue.clone());
    }
    if let Some(fog_enable) = &attributes.fog_enable {
        packet.add_string(50u16, fog_enable.clone());
    }
    if let Some(fog_green) = &attributes.fog_green {
        packet.add_string(51u16, fog_green.clone());
    }
    if let Some(fog_maximum) = &attributes.fog_maximum {
        packet.add_string(52u16, fog_maximum.clone());
    }
    if let Some(fog_minimum) = &attributes.fog_minimum {
        packet.add_string(53u16, fog_minimum.clone());
    }
    if let Some(fog_red) = &attributes.fog_red {
        packet.add_string(54u16, fog_red.clone());
    }
    if let Some(gravity) = &attributes.gravity {
        packet.add_string(55u16, gravity.clone());
    }
    if let Some(ground) = &attributes.ground {
        packet.add_string(56u16, ground.clone());
    }
    if let Some(home_page) = &attributes.home_page {
        packet.add_string(57u16, home_page.clone());
    }
    if let Some(keywords) = &attributes.keywords {
        packet.add_string(58u16, keywords.clone());
    }
    if let Some(light_blue) = &attributes.light_blue {
        packet.add_string(59u16, light_blue.clone());
    }
    if let Some(light_draw_bright) = &attributes.light_draw_bright {
        packet.add_string(60u16, light_draw_bright.clone());
    }
    if let Some(light_draw_front) = &attributes.light_draw_front {
        packet.add_string(61u16, light_draw_front.clone());
    }
    if let Some(light_draw_size) = &attributes.light_draw_size {
        packet.add_string(62u16, light_draw_size.clone());
    }
    if let Some(light_green) = &attributes.light_green {
        packet.add_string(63u16, light_green.clone());
    }
    if let Some(light_mask) = &attributes.light_mask {
        packet.add_string(64u16, light_mask.clone());
    }
    if let Some(light_red) = &attributes.light_red {
        packet.add_string(65u16, light_red.clone());
    }
    if let Some(light_texture) = &attributes.light_texture {
        packet.add_string(66u16, light_texture.clone());
    }
    if let Some(light_x) = &attributes.light_x {
        packet.add_string(67u16, light_x.clone());
    }
    if let Some(light_y) = &attributes.light_y {
        packet.add_string(68u16, light_y.clone());
    }
    if let Some(light_z) = &attributes.light_z {
        packet.add_string(69u16, light_z.clone());
    }
    if let Some(max_light_radius) = &attributes.max_light_radius {
        packet.add_string(70u16, max_light_radius.clone());
    }
    if let Some(max_users) = &attributes.max_users {
        packet.add_string(71u16, max_users.clone());
    }
    if let Some(minimum_visibility) = &attributes.minimum_visibility {
        packet.add_string(72u16, minimum_visibility.clone());
    }
    if let Some(object_count) = &attributes.object_count {
        packet.add_string(73u16, object_count.clone());
    }
    if let Some(object_password) = &attributes.object_password {
        let mut encryptor = PkwareDcl::new();
        let encrypted_object_password = encryptor.encrypt(object_password.as_bytes());
        packet.add_data(74u16, encrypted_object_password);
    }
    if let Some(object_path) = &attributes.object_path {
        packet.add_string(76u16, object_path.clone());
    }
    if let Some(object_refresh) = &attributes.object_refresh {
        packet.add_string(77u16, object_refresh.clone());
    }
    if let Some(public_speaker_right) = &attributes.public_speaker_right {
        packet.add_string(78u16, public_speaker_right.clone());
    }
    if let Some(rating) = &attributes.rating {
        packet.add_string(79u16, rating.clone());
    }
    if let Some(repeating_ground) = &attributes.repeating_ground {
        packet.add_string(80u16, repeating_ground.clone());
    }
    if let Some(restricted_radius) = &attributes.restricted_radius {
        packet.add_string(81u16, restricted_radius.clone());
    }
    if let Some(size) = &attributes.size {
        packet.add_string(82u16, size.clone());
    }
    if let Some(skybox) = &attributes.skybox {
        packet.add_string(83u16, skybox.clone());
    }
    if let Some(sky_bottom_blue) = &attributes.sky_bottom_blue {
        packet.add_string(84u16, sky_bottom_blue.clone());
    }
    if let Some(sky_bottom_green) = &attributes.sky_bottom_green {
        packet.add_string(85u16, sky_bottom_green.clone());
    }
    if let Some(sky_bottom_red) = &attributes.sky_bottom_red {
        packet.add_string(86u16, sky_bottom_red.clone());
    }
    if let Some(sky_east_blue) = &attributes.sky_east_blue {
        packet.add_string(87u16, sky_east_blue.clone());
    }
    if let Some(sky_east_green) = &attributes.sky_east_green {
        packet.add_string(88u16, sky_east_green.clone());
    }
    if let Some(sky_east_red) = &attributes.sky_east_red {
        packet.add_string(89u16, sky_east_red.clone());
    }
    if let Some(sky_north_blue) = &attributes.sky_north_blue {
        packet.add_string(90u16, sky_north_blue.clone());
    }
    if let Some(sky_north_green) = &attributes.sky_north_green {
        packet.add_string(91u16, sky_north_green.clone());
    }
    if let Some(sky_north_red) = &attributes.sky_north_red {
        packet.add_string(92u16, sky_north_red.clone());
    }
    if let Some(sky_south_blue) = &attributes.sky_south_blue {
        packet.add_string(93u16, sky_south_blue.clone());
    }
    if let Some(sky_south_green) = &attributes.sky_south_green {
        packet.add_string(94u16, sky_south_green.clone());
    }
    if let Some(sky_south_red) = &attributes.sky_south_red {
        packet.add_string(95u16, sky_south_red.clone());
    }
    if let Some(sky_top_blue) = &attributes.sky_top_blue {
        packet.add_string(96u16, sky_top_blue.clone());
    }
    if let Some(sky_top_green) = &attributes.sky_top_green {
        packet.add_string(97u16, sky_top_green.clone());
    }
    if let Some(sky_top_red) = &attributes.sky_top_red {
        packet.add_string(98u16, sky_top_red.clone());
    }
    if let Some(sky_west_blue) = &attributes.sky_west_blue {
        packet.add_string(99u16, sky_west_blue.clone());
    }
    if let Some(sky_west_green) = &attributes.sky_west_green {
        packet.add_string(100u16, sky_west_green.clone());
    }
    if let Some(sky_west_red) = &attributes.sky_west_red {
        packet.add_string(101u16, sky_west_red.clone());
    }
    if let Some(sound_footstep) = &attributes.sound_footstep {
        packet.add_string(102u16, sound_footstep.clone());
    }
    if let Some(sound_water_enter) = &attributes.sound_water_enter {
        packet.add_string(103u16, sound_water_enter.clone());
    }
    if let Some(sound_water_exit) = &attributes.sound_water_exit {
        packet.add_string(104u16, sound_water_exit.clone());
    }
    if let Some(speak_right) = &attributes.speak_right {
        packet.add_string(105u16, speak_right.clone());
    }
    if let Some(special_commands_right) = &attributes.special_commands_right {
        packet.add_string(106u16, special_commands_right.clone());
    }
    if let Some(special_objects_right) = &attributes.special_objects_right {
        packet.add_string(107u16, special_objects_right.clone());
    }
    if let Some(terrain_ambient) = &attributes.terrain_ambient {
        packet.add_string(108u16, terrain_ambient.clone());
    }
    if let Some(terrain_diffuse) = &attributes.terrain_diffuse {
        packet.add_string(109u16, terrain_diffuse.clone());
    }
    if let Some(terrain_offset) = &attributes.terrain_offset {
        packet.add_string(110u16, terrain_offset.clone());
    }
    if let Some(terrain_timestamp) = &attributes.terrain_timestamp {
        packet.add_string(111u16, terrain_timestamp.clone());
    }
    if let Some(title) = &attributes.title {
        packet.add_string(112u16, title.clone());
    }
    if let Some(voip_right) = &attributes.voip_right {
        packet.add_string(113u16, voip_right.clone());
    }
    if let Some(water_blue) = &attributes.water_blue {
        packet.add_string(114u16, water_blue.clone());
    }
    if let Some(water_bottom_mask) = &attributes.water_bottom_mask {
        packet.add_string(115u16, water_bottom_mask.clone());
    }
    if let Some(water_bottom_texture) = &attributes.water_bottom_texture {
        packet.add_string(116u16, water_bottom_texture.clone());
    }
    if let Some(water_enabled) = &attributes.water_enabled {
        packet.add_string(117u16, water_enabled.clone());
    }
    if let Some(water_green) = &attributes.water_green {
        packet.add_string(118u16, water_green.clone());
    }
    if let Some(water_level) = &attributes.water_level {
        packet.add_string(119u16, water_level.clone());
    }
    if let Some(water_mask) = &attributes.water_mask {
        packet.add_string(120u16, water_mask.clone());
    }
    if let Some(water_opacity) = &attributes.water_opacity {
        packet.add_string(121u16, water_opacity.clone());
    }
    if let Some(water_red) = &attributes.water_red {
        packet.add_string(122u16, water_red.clone());
    }
    if let Some(water_speed) = &attributes.water_speed {
        packet.add_string(123u16, water_speed.clone());
    }
    if let Some(water_surface_move) = &attributes.water_surface_move {
        packet.add_string(124u16, water_surface_move.clone());
    }
    if let Some(water_texture) = &attributes.water_texture {
        packet.add_string(125u16, water_texture.clone());
    }
    if let Some(water_under_terrain) = &attributes.water_under_terrain {
        packet.add_string(126u16, water_under_terrain.clone());
    }
    if let Some(water_visibility) = &attributes.water_visibility {
        packet.add_string(127u16, water_visibility.clone());
    }
    if let Some(water_wave_move) = &attributes.water_wave_move {
        packet.add_string(128u16, water_wave_move.clone());
    }
    if let Some(welcome_message) = &attributes.welcome_message {
        packet.add_string(129u16, welcome_message.clone());
    }
    if let Some(disable_multiple_media) = &attributes.disable_multiple_media {
        packet.add_string(130u16, disable_multiple_media.clone());
    }
    if let Some(sound_ambient) = &attributes.sound_ambient {
        packet.add_string(131u16, sound_ambient.clone());
    }
    if let Some(botmenu_url) = &attributes.botmenu_url {
        packet.add_string(132u16, botmenu_url.clone());
    }
    if let Some(enable_bump_event) = &attributes.enable_bump_event {
        packet.add_string(133u16, enable_bump_event.clone());
    }
    if let Some(enable_sync_events) = &attributes.enable_sync_events {
        packet.add_string(134u16, enable_sync_events.clone());
    }
    if let Some(enable_cav) = &attributes.enable_cav {
        packet.add_string(135u16, enable_cav.clone());
    }
    if let Some(enable_pav) = &attributes.enable_pav {
        packet.add_string(136u16, enable_pav.clone());
    }
    if let Some(friction) = &attributes.friction {
        packet.add_string(137u16, friction.clone());
    }
    if let Some(water_friction) = &attributes.water_friction {
        packet.add_string(138u16, water_friction.clone());
    }
    if let Some(slopeslide_enabled) = &attributes.slopeslide_enabled {
        packet.add_string(139u16, slopeslide_enabled.clone());
    }
    if let Some(slopeslide_min_angle) = &attributes.slopeslide_min_angle {
        packet.add_string(140u16, slopeslide_min_angle.clone());
    }
    if let Some(slopeslide_max_angle) = &attributes.slopeslide_max_angle {
        packet.add_string(141u16, slopeslide_max_angle.clone());
    }
    if let Some(fog_tinted) = &attributes.fog_tinted {
        packet.add_string(142u16, fog_tinted.clone());
    }
    if let Some(light_source_use_color) = &attributes.light_source_use_color {
        packet.add_string(143u16, light_source_use_color.clone());
    }
    if let Some(light_source_color) = &attributes.light_source_color {
        packet.add_string(144u16, light_source_color.clone());
    }
    if let Some(chat_disable_url_clicks) = &attributes.chat_disable_url_clicks {
        packet.add_string(145u16, chat_disable_url_clicks.clone());
    }
    if let Some(mover_empty_reset_timeout) = &attributes.mover_empty_reset_timeout {
        packet.add_string(146u16, mover_empty_reset_timeout.clone());
    }
    if let Some(mover_used_reset_timeout) = &attributes.mover_used_reset_timeout {
        packet.add_string(147u16, mover_used_reset_timeout.clone());
    }
    if let Some(v4_objects_right) = &attributes.v4_objects_right {
        packet.add_string(148u16, v4_objects_right.clone());
    }
    if let Some(disable_shadows) = &attributes.disable_shadows {
        packet.add_string(149u16, disable_shadows.clone());
    }
    if let Some(enable_camera_collision) = &attributes.enable_camera_collision {
        packet.add_string(150u16, enable_camera_collision.clone());
    }
    if let Some(special_commands) = &attributes.special_commands {
        packet.add_string(151u16, special_commands.clone());
    }
    if let Some(cav_object_path) = &attributes.cav_object_path {
        packet.add_string(152u16, cav_object_path.clone());
    }
    if let Some(cav_object_password) = &attributes.cav_object_password {
        let mut decryptor = PkwareDcl::new();
        let decrypted_cav_object_password = decryptor.decrypt(cav_object_password.as_bytes());
        packet.add_data(153u16, decrypted_cav_object_password);
    }
    if let Some(cav_object_refresh) = &attributes.cav_object_refresh {
        packet.add_string(154u16, cav_object_refresh.clone());
    }
    if let Some(terrain_right) = &attributes.terrain_right {
        packet.add_string(155u16, terrain_right.clone());
    }
    if let Some(voip_conference_global) = &attributes.voip_conference_global {
        packet.add_string(156u16, voip_conference_global.clone());
    }
    if let Some(voip_moderate_global) = &attributes.voip_moderate_global {
        packet.add_string(157u16, voip_moderate_global.clone());
    }
    if let Some(camera_zoom) = &attributes.camera_zoom {
        packet.add_string(158u16, camera_zoom.clone());
    }
    if let Some(wait_limit) = &attributes.wait_limit {
        packet.add_string(159u16, wait_limit.clone());
    }
    if let Some(voipcast_host) = &attributes.voipcast_host {
        packet.add_string(160u16, voipcast_host.clone());
    }
    if let Some(voipcast_port) = &attributes.voipcast_port {
        packet.add_string(161u16, voipcast_port.clone());
    }
    if let Some(enable_wireframe) = &attributes.enable_wireframe {
        packet.add_string(162u16, enable_wireframe.clone());
    }

    if let Some(world) = &mut instance.world {
        world.connection.send(packet);
        SdkResult::Ok(())
    } else {
        SdkResult::Err(SdkError::NotConnectedToWorld)
    }
}
