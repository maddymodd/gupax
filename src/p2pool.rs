// Gupax - GUI Uniting P2Pool And XMRig
//
// Copyright (c) 2022 hinto-janaiyo
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{
	App,
	Regexes,
	constants::*,
	disk::*,
	node::*
};
use egui::{
	TextEdit,SelectableLabel,ComboBox,Label,FontId,Button,Color32,RichText,Slider,Checkbox,
	TextStyle::*,
	FontFamily::Proportional,
	TextBuffer,
};
use std::sync::{Arc,Mutex};
use std::thread;
use regex::Regex;
use log::*;

impl P2pool {
	pub fn show(&mut self, node_vec: &mut Vec<(String, Node)>, og: &Arc<Mutex<State>>, online: bool, ping: &Arc<Mutex<Ping>>, regex: &Regexes, width: f32, height: f32, ctx: &egui::Context, ui: &mut egui::Ui) {
	let text_edit = height / 22.0;
	//---------------------------------------------------------------------------------------------------- Console
	ui.group(|ui| {
		let height = height / SPACE;
		let width = width - SPACE;
		ui.style_mut().override_text_style = Some(Monospace);
		ui.add_sized([width, height*3.0], TextEdit::multiline(&mut "".to_string()));
		ui.add_sized([width, text_edit], TextEdit::hint_text(TextEdit::singleline(&mut "".to_string()), r#"Type a command (e.g "help" or "status") and press Enter"#));
	});

	//---------------------------------------------------------------------------------------------------- Args
	if ! self.simple {
	ui.group(|ui| { ui.horizontal(|ui| {
		let width = (width/10.0) - SPACE;
		ui.style_mut().override_text_style = Some(Monospace);
		ui.add_sized([width, text_edit], Label::new("Command arguments:"));
		ui.add_sized([ui.available_width(), text_edit], TextEdit::hint_text(TextEdit::singleline(&mut self.arguments), r#"--wallet <...> --host <...>"#)).on_hover_text(P2POOL_COMMAND);
		self.arguments.truncate(1024);
	})});
	ui.set_enabled(self.arguments.is_empty());
	}

	//---------------------------------------------------------------------------------------------------- Address
	ui.group(|ui| {
		let width = width - SPACE;
		ui.spacing_mut().text_edit_width = (width)-(SPACE*3.0);
		ui.style_mut().override_text_style = Some(Monospace);
		let text;
		let color;
		let len = format!("{:02}", self.address.len());
		if self.address.is_empty() {
			text = format!("Monero Address [{}/95] ➖", len);
			color = Color32::LIGHT_GRAY;
		} else if self.address.len() == 95 && Regex::is_match(&regex.address, &self.address) && ! self.address.contains("0") && ! self.address.contains("O") && ! self.address.contains("l") {
			text = format!("Monero Address [{}/95] ✔", len);
			color = Color32::from_rgb(100, 230, 100);
		} else {
			text = format!("Monero Address [{}/95] ❌", len);
			color = Color32::from_rgb(230, 50, 50);
		}
		ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
		ui.add_sized([width, text_edit], TextEdit::hint_text(TextEdit::singleline(&mut self.address), "4...")).on_hover_text(P2POOL_ADDRESS);
		self.address.truncate(95);
	});

	//---------------------------------------------------------------------------------------------------- Simple
	let height = ui.available_height();
	if self.simple {
		// [Node]
		let height = height / 6.0;
		ui.spacing_mut().slider_width = width - 8.0;
		ui.spacing_mut().icon_width = width / 25.0;
		ui.vertical(|ui| {
		ui.horizontal(|ui| {
			// [Ping List]
			let id = self.node;
			let ip = enum_to_ip(id);
			let mut ms = 0;
			let mut color = Color32::LIGHT_GRAY;
			for data in ping.lock().unwrap().nodes.iter() {
				if data.id == id {
					ms = data.ms;
					color = data.color;
					break
				}
			}
			let text = RichText::new(format!(" ⏺ {}ms | {} | {}", ms, id, ip)).color(color);
			ComboBox::from_id_source("nodes").selected_text(RichText::text_style(text, Monospace)).show_ui(ui, |ui| {
				for data in ping.lock().unwrap().nodes.iter() {
					let ms = crate::node::format_ms(data.ms);
					let id = crate::node::format_enum(data.id);
					let text = RichText::text_style(RichText::new(format!(" ⏺ {} | {} | {}", ms, id, data.ip)).color(data.color), Monospace);
					ui.selectable_value(&mut self.node, data.id, text);
				}
			});
		});

		ui.add_space(5.0);

		ui.horizontal(|ui| {
		let width = (width/2.0)-4.0;
		// [Select fastest node]
		if ui.add_sized([width, height], Button::new("Select fastest node")).on_hover_text(P2POOL_SELECT_FASTEST).clicked() {
			let pinged = ping.lock().unwrap().pinged;
			let fastest = ping.lock().unwrap().fastest;
			if pinged && og.lock().unwrap().p2pool.node != fastest {
				og.lock().unwrap().p2pool.node = ping.lock().unwrap().fastest;
				og.lock().unwrap().save();
			}
		}
		// [Ping Button]
		ui.set_enabled(!ping.lock().unwrap().pinging);
		if ui.add_sized([width, height], Button::new("Ping community nodes")).on_hover_text(P2POOL_PING).clicked() {
			let ping = Arc::clone(&ping);
			let og = Arc::clone(og);
			thread::spawn(move|| {
				info!("Spawning ping thread...");
				crate::node::ping(ping, og);
			});
		}});

		ui.vertical(|ui| {
			let height = height / 2.0;
			let pinging = ping.lock().unwrap().pinging;
			ui.set_enabled(pinging);
			let prog = ping.lock().unwrap().prog.round();
			let msg = RichText::text_style(RichText::new(format!("{} ... {}%", ping.lock().unwrap().msg, prog)), Monospace);
			let height = height / 1.25;
			ui.add_space(5.0);
			ui.add_sized([width, height], Label::new(msg));
			ui.add_space(5.0);
			if pinging {
				ui.add_sized([width, height], egui::Spinner::new().size(height));
			} else {
				ui.add_sized([width, height], egui::Label::new("..."));
			}
			ui.add_sized([width, height], egui::ProgressBar::new(prog.round()/100.0));
			ui.add_space(5.0);
		});
		});

		ui.group(|ui| {
		ui.horizontal(|ui| {
			let width = (width/2.0)-(SPACE*1.75);
			// [Auto-node] + [Auto-select]
			let mut style = (*ctx.style()).clone();
			style.spacing.icon_width_inner = height/1.5;
			style.spacing.icon_width = height;
			style.spacing.icon_spacing = 20.0;
			ctx.set_style(style);
			ui.add_sized([width, height], egui::Checkbox::new(&mut self.auto_select, "Auto-select")).on_hover_text(P2POOL_AUTO_SELECT);
			ui.separator();
			ui.add_sized([width, height], egui::Checkbox::new(&mut self.auto_node, "Auto-node")).on_hover_text(P2POOL_AUTO_NODE);
		})});

	//---------------------------------------------------------------------------------------------------- Advanced
	} else {
		let mut incorrect_input = false; // This will disable [Add/Delete] on bad input
		// [Monero node IP/RPC/ZMQ]
		ui.horizontal(|ui| {
		ui.group(|ui| {
			let width = width/10.0;
			ui.vertical(|ui| {
			ui.style_mut().override_text_style = Some(Monospace);
			ui.spacing_mut().text_edit_width = width*3.32;
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:02}", self.name.len());
				if self.name.is_empty() {
					text = format!("Name [ {}/30 ]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if Regex::is_match(&regex.name, &self.name) {
					text = format!("Name [ {}/30 ]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!("Name [ {}/30 ]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.name).on_hover_text(P2POOL_NAME);
				self.name.truncate(30);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = format!("{:03}", self.ip.len());
				if self.ip.is_empty() {
					text = format!("  IP [{}/255]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if self.ip == "localhost" || Regex::is_match(&regex.ipv4, &self.ip) || Regex::is_match(&regex.domain, &self.ip) {
					text = format!("  IP [{}/255]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!("  IP [{}/255]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.ip).on_hover_text(P2POOL_NODE_IP);
				self.ip.truncate(255);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = self.rpc.len();
				if self.rpc.is_empty() {
					text = format!(" RPC [  {}/5  ]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if Regex::is_match(&regex.port, &self.rpc) {
					text = format!(" RPC [  {}/5  ]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!(" RPC [  {}/5  ]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.rpc).on_hover_text(P2POOL_RPC_PORT);
				self.rpc.truncate(5);
			});
			ui.horizontal(|ui| {
				let text;
				let color;
				let len = self.zmq.len();
				if self.zmq.is_empty() {
					text = format!(" ZMQ [  {}/5  ]➖", len);
					color = Color32::LIGHT_GRAY;
					incorrect_input = true;
				} else if Regex::is_match(&regex.port, &self.zmq) {
					text = format!(" ZMQ [  {}/5  ]✔", len);
					color = Color32::from_rgb(100, 230, 100);
				} else {
					text = format!(" ZMQ [  {}/5  ]❌", len);
					color = Color32::from_rgb(230, 50, 50);
					incorrect_input = true;
				}
				ui.add_sized([width, text_edit], Label::new(RichText::new(text).color(color)));
				ui.text_edit_singleline(&mut self.zmq).on_hover_text(P2POOL_ZMQ_PORT);
				self.zmq.truncate(5);
			});
		});

		ui.vertical(|ui| {
			let width = ui.available_width();
			ui.add_space(1.0);
			// [Manual node selection]
			ui.spacing_mut().slider_width = width - 8.0;
			ui.spacing_mut().icon_width = width / 25.0;
			// [Ping List]
			let text = RichText::new(format!("{}. {} | {}", self.selected_index, self.selected_name, self.selected_ip));
			ComboBox::from_id_source("nodes").selected_text(RichText::text_style(text, Monospace)).show_ui(ui, |ui| {
				let mut n = 1;
				for (name, node) in node_vec.iter() {
					let text = RichText::text_style(RichText::new(format!("{}. {}\n     IP: {}\n    RPC: {}\n    ZMQ: {}", n, name, node.ip, node.rpc, node.zmq)), Monospace);
					if ui.add(SelectableLabel::new(self.selected_name == *name, text)).clicked() {
						self.selected_index = n;
						self.selected_name = name.clone();
					}
//					ui.selectable_value(&mut self.selected_name, name.clone(), text);
					n += 1;
				}
			});
			// [Add] + [Delete]
			let node_vec_len = node_vec.len();
			ui.horizontal(|ui| {
				let mut exists = false;
				for (name, _) in node_vec.iter() {
					if *name == self.name { exists = true; }
				}
				ui.set_enabled(!incorrect_input && !exists && node_vec_len < 1000);
				let text = format!("{}\n    Currently selected node: {}. {}\n    Current amount of nodes: {}/1000", P2POOL_ADD, self.selected_index, self.selected_name, node_vec_len);
				if ui.add_sized([width, text_edit], Button::new("Add")).on_hover_text(text).clicked() {
					let node = Node {
						ip: self.ip.clone(),
						rpc: self.rpc.clone(),
						zmq: self.zmq.clone(),
					};
					node_vec.push((self.name.clone(), node));
					info!("Node | Added [index: {}, name: \"{}\", ip: \"{}\", rpc: {}, zmq: {}]", node_vec_len+1, self.name, self.ip, self.rpc, self.zmq);
				}
			});
			ui.horizontal(|ui| {
				ui.set_enabled(node_vec_len > 1);
				let text = format!("{}\n    Currently selected node: {}. {}\n    Current amount of nodes: {}/1000", P2POOL_ADD, self.selected_index, self.selected_name, node_vec_len);
				if ui.add_sized([width, text_edit], Button::new("Delete")).on_hover_text(text).clicked() {
					let mut n = 0;
					for (name, _) in node_vec.iter() {
						if *name == self.selected_name {
							// If deleting [0], make selected = [1]
							// instead of attempting to [0-1] (panic!)
							match n {
								0 => { self.selected_name = node_vec[1].0.clone(); self.selected_index = 1; },
								_ => { self.selected_name = node_vec[n-1].0.clone(); self.selected_index = n as u16; },
							};
							node_vec.remove(n);
							info!("Node | Deleted [index: {}, name: \"{}\", ip: \"{}\", rpc: {}, zmq: {}]", n+1, self.selected_name, self.selected_ip, self.selected_rpc, self.selected_zmq);
							break
						}
						n += 1;
					}
				}
			});
			ui.horizontal(|ui| {
				ui.set_enabled(!self.name.is_empty() || !self.ip.is_empty() || !self.rpc.is_empty() || !self.zmq.is_empty());
				if ui.add_sized([width, text_edit], Button::new("Clear")).on_hover_text(P2POOL_CLEAR).clicked() {
					self.name.clear();
					self.ip.clear();
					self.rpc.clear();
					self.zmq.clear();
				}
			});
		});
		});
		});
		ui.add_space(5.0);

		// [Main/Mini]
		ui.horizontal(|ui| {
		let height = height/3.0;
		ui.group(|ui| { ui.horizontal(|ui| {
			let width = (width/4.0)-SPACE;
			let height = height + 6.0;
			if ui.add_sized([width, height], SelectableLabel::new(self.mini == false, "P2Pool Main")).on_hover_text(P2POOL_MAIN).clicked() { self.mini = false; }
			if ui.add_sized([width, height], SelectableLabel::new(self.mini == true, "P2Pool Mini")).on_hover_text(P2POOL_MINI).clicked() { self.mini = true; }
		})});
		// [Out/In Peers] + [Log Level]
		ui.group(|ui| { ui.vertical(|ui| {
			let text = (ui.available_width()/10.0)-SPACE;
			let width = (text*8.0)-SPACE;
			let height = height/3.0;
			ui.style_mut().spacing.slider_width = width/1.2;
			ui.style_mut().spacing.interact_size.y = height;
			ui.style_mut().override_text_style = Some(Name("MonospaceSmall".into()));
//			ui.style_mut().override_text_style = Some(Monospace);
			ui.horizontal(|ui| {
				ui.add_sized([text, height], Label::new("Out peers [10-450]:"));
				ui.add_sized([width, height], Slider::new(&mut self.out_peers, 10..=450)).on_hover_text(P2POOL_OUT);
				ui.add_space(ui.available_width()-4.0);
			});
			ui.horizontal(|ui| {
				ui.add_sized([text, height], Label::new(" In peers [10-450]:"));
				ui.add_sized([width, height], Slider::new(&mut self.in_peers, 10..=450)).on_hover_text(P2POOL_IN);
			});
			ui.horizontal(|ui| {
				ui.add_sized([text, height], Label::new("   Log level [0-6]:"));
				ui.add_sized([width, height], Slider::new(&mut self.log_level, 0..=6)).on_hover_text(P2POOL_LOG);
			});
		})});
		});
	}
	}
}
