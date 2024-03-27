pub fn get_banner() -> String {
    let banner = concat!(
    "    _  _____ _    _      ___ __  __     _   _    _    \r\n", 
    "   | |/ |_ _| |  | |    | __|  \\/  |   /_\\ | |  | |   \r\n", 
    "   | ' < | || |__| |__  | _|| |\\/| |  / _ \\| |__| |__ \r\n", 
    "   |_|\\_|___|____|____| |___|_|  |_| /_/ \\_|____|____|\r\n");
    banner.to_owned()
}

pub fn get_bye() -> String {
    concat!(
     "    ________      ___    ___ _______   ___  ___       \r\n",
     "   |\\   __  \\    |\\  \\  /  /|\\  ___ \\ |\\  \\|\\  \\      \r\n", 
     "   \\ \\  \\|\\ /_   \\ \\  \\/  / | \\   __/|\\ \\  \\ \\  \\     \r\n", 
     "    \\ \\   __  \\   \\ \\    / / \\ \\  \\_|/_\\ \\  \\ \\  \\    \r\n", 
     "     \\ \\  \\|\\  \\   \\/  /  /   \\ \\  \\_|\\ \\ \\__\\ \\__\\   \r\n", 
     "      \\ \\_______\\__/  / /      \\ \\_______\\|__|\\|__|   \r\n", 
     "       \\|_______|\\___/ /        \\|_______|   ___  ___ \r\n", 
     "                \\|___|/                     |\\__\\|\\__\\\r\n", 
     "                                            \\|__|\\|__|\r\n").to_owned()
}

pub fn get_bottom_tips() -> String {
    "[q] to Exit. Type \"-help\" to review the guide.".to_owned()
}

pub fn get_help()->String{
    let content = r#"
    Controls:
     - [q] to quit. [f]/[b] jump to next/previous 10 options. 
     - [j]/[Arrow Down] select next 1 option. [k]/[Arrow Up] select previous 1 option 
     will add more controls in the future
    Input:
     - "-open" open the dir with finder/explorer.
     - "-config" open the config dir with finder/explorer.
    "#;
    return content.to_owned();
}