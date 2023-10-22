use std::collections::HashMap;
use std::path::PathBuf;
use iced::{Application, Command, Element, Length, Renderer, Settings, Theme, theme};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{text, image, pick_list};
use iced::widget::{Column, row, scrollable, column, button};
use iced::widget::scrollable::Properties;
use iced::widget::vertical_space;
use std::ops::Not;
use iced::widget::image::Handle;
use crate::ftb_modpacks::{FTBModpackList};

mod manifest;
mod ftb_pack;
mod ftb_modpacks;

#[tokio::main]
async fn main() -> iced::Result {
    // dbg!(ftb_modpacks::FTBModpackList::get_all().await);
    // if let Ok(modpacks) = ftb_modpacks::FTBModpackList::from_file("modpacks.json") {
    //     modpacks.packs.iter().enumerate().for_each(|(i, pack)| {
    //         println!("[{i:>3}] - {}", pack.name);
    //     });
        
    //     let sel = modpacks.packs.first().unwrap();
    //     let pack: Result<manifest::Manifest, _> = ftb_pack::Pack::get_from_id(sel.id, sel.versions.first().unwrap().id)
    //         .await
    //         .unwrap()
    //         .try_into();
    //     dbg!(pack);
    // }
    
    // let _ = std::fs::create_dir("modpack");
    // let path = "./pack.json";
    // let raw = std::fs::read_to_string(&path).unwrap();
    // let json: ftb_pack::Pack = serde_json::from_str(&raw).unwrap();
    // let manifest = manifest::Manifest::try_from(json.clone()).unwrap();
    
    // for file in json.files {
    //     ftb_pack::get_overrides("modpack", &file).await;
    // }
    
    // let ftb_modpacks = FTBModpackList::from_file("modpacks.json").unwrap();
    let ftb_modpacks = FTBModpackList::get_all().await.unwrap();
    App::run(
        Settings {
            flags: ftb_modpacks,
            ..Default::default()
        }
    )
}

#[derive(Debug, Clone)]
pub enum Message {
    Scrolled(scrollable::Viewport),
    ModpackSelected(usize),
    LogoLoaded(String, Vec<u8>),
    OpenProjectSite(i64, String),
    Version(String),
    DownloadClient(i64),
    FTBModList(Box<Result<ftb_pack::Pack, ()>>, String),
    DownloadComplete,
}

struct App {
    modpack_list: FTBModpackList,
    selected: Option<ftb_modpacks::Modpack>,
    logos: HashMap<String, Box<Handle>>,
    selected_version: Option<String>,
    is_downloading: bool,
    scroll_offset: scrollable::RelativeOffset,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Message = Message;
    type Flags = FTBModpackList;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            App {
                modpack_list: flags,
                selected: None,
                logos: HashMap::new(),
                selected_version: None,
                is_downloading: false,
                scroll_offset: scrollable::RelativeOffset::START,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "FTB 2 Modpack".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Scrolled(offset) => {
                self.scroll_offset = offset.relative_offset();
            }
            Message::ModpackSelected(index) => {
                if self.is_downloading {
                    return Command::none();
                }

                if let Some(sel) = self.modpack_list.packs.get(index) {
                    self.selected = Some(sel.clone());
                    self.selected_version = sel.versions.first().map(|v| v.name.clone());
                    if let Some(logo) = &sel.art.logo {
                        if !self.logos.contains_key(logo) {
                            return Command::perform(get_image(logo.clone()), |(id, n)| Message::LogoLoaded(id, n));
                        }
                    }
                }
            }
            Message::LogoLoaded(id, bytes) => {
                let img = image::Handle::from_memory(bytes);
                self.logos.insert(id, Box::new(img));
            },
            Message::OpenProjectSite(id, slug) => {
                let _ = open::that_detached(format!("https://www.feed-the-beast.com/modpacks/{id}-{slug}"));
            },
            Message::Version(version) => {
                self.selected_version = Some(version);
            },
            Message::DownloadClient(id) => {
                if let Some(version) = &self.selected_version {
                    if let Some(modpack) = &self.selected {
                        if let Some(version_id) =  modpack.versions.iter().find(|ver| ver.name.eq(version)).map(|ver| ver.id) {
                            let n = modpack.name.clone();
                            return Command::perform(
                                ftb_pack::Pack::get_from_id(id, version_id),
                                |pack| Message::FTBModList(Box::new(pack), n)
                            );
                        }
                    }
                }
            },
            Message::FTBModList(pack, name) => {
                if let Ok(pack) = *pack {
                    if let Some(base_dir) = rfd::FileDialog::new().pick_folder() {
                        self.is_downloading = true;
                        return Command::perform(download_client(base_dir, pack, name.clone()), |_| Message::DownloadComplete);
                    }
                }
            },
            Message::DownloadComplete => {
                self.is_downloading = false;
            },
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let list: Element<Message> = scrollable(
            self.modpack_list.packs.iter().enumerate().fold(
                Column::new()
                    .width(Length::Fill)
                    .height(Length::Fill),
                |c,(i, e)| {
                    c.push(modpack_button(i, e))
                })
        )
            .height(Length::Fill)
            .direction(scrollable::Direction::Vertical(
                Properties::new()
                    .width(10)
                    .margin(2)
                    .scroller_width(10)
                    .alignment(scrollable::Alignment::Start)
            ))
            .on_scroll(Message::Scrolled)
            .width(Length::FillPortion(40))
            .into();

         let selected = column![
            selected_modpack(&self.selected_version, self.is_downloading, &self.logos, &self.selected)
        ].width(Length::FillPortion(60));

        row![
            list,
            selected
        ].padding(4).into()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}

async fn download_client(base_dir: PathBuf, pack: ftb_pack::Pack, name: String) {
    let mut work_dir = base_dir.clone();

    work_dir.push(format!("{} {}", name, pack.name));
    let _ = std::fs::create_dir(work_dir.clone());

    let manifest = manifest::Manifest::try_from(pack.clone()).unwrap();
    manifest::save_manifest(work_dir.clone(), manifest.clone());

    for file in &pack.files {
        ftb_pack::get_overrides(work_dir.clone(), file).await;
    }
}

async fn get_image(url: String) -> (String, Vec<u8>) {
    let bytes = match reqwest::get(url.clone()).await {
        Err(_) => None,
        Ok(res) => res.bytes().await.ok()
    };

    (url, bytes.map(|b|b.to_vec()).unwrap_or(vec![]))
}

fn selected_modpack<'a>(selected_version: &Option<String>, is_downloading: bool, logos: &'a HashMap<String, Box<Handle>>, pack: &Option<ftb_modpacks::Modpack>) -> Element<'a, Message> {
    match pack {
        None => row!().padding(10).into(),
        Some(pack) => {
            let img: Element<'_, Message> = if let Some(a) = pack.art.logo.clone() {
                if let Some(logo) = logos.get(&a) {
                    image::Image::new(*logo.clone()).width(Length::FillPortion(40)).into()
                } else {
                    text("No logo available".to_string())
                        .width(256)
                        .height(256)
                        .vertical_alignment(Vertical::Center)
                        .horizontal_alignment(Horizontal::Center)
                        .into()
                }
            } else {
                text("No logo available".to_string())
                    .width(256)
                    .height(256)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center)
                    .into()
            };

            let versions = pick_list(
                pack.versions.clone().iter().map(|v| v.name.clone()).collect::<Vec<_>>(),
                selected_version.clone(),Message::Version
            );


            column![
                row![
                    img,
                    column![
                        text(pack.name.to_string()).size(24),
                        button("Project site").padding(8).on_press(Message::OpenProjectSite(pack.id, pack.slug.clone()))
                    ].width(Length::FillPortion(60)).spacing(8)
                ].padding(10).spacing(8),

                row![
                    text(pack.synopsis.clone()).size(18)
                ].padding(10),

                vertical_space(Length::Fill),

                row![
                    versions,
                    button("Client")
                        .style(
                            if is_downloading {
                                theme::Button::Secondary
                            } else {
                                theme::Button::Primary
                        })
                    .on_press_maybe(is_downloading.not().then_some(Message::DownloadClient(pack.id))).padding(10)
                ].padding(10).spacing(8)
            ].into()
        },
    }
}

fn modpack_button(index: usize, pack: &ftb_modpacks::Modpack) -> Element<'_, Message> {
    let label= &pack.name;
    row!(
        button(&**label)
        .width(Length::Fill)
        .padding(2)
        .on_press(Message::ModpackSelected(index))
    ).into()
}

