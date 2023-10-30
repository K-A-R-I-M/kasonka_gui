export interface Item {
	title: string;
	duration: string;
}

export interface Notif{
	id: number,
	audio_title: string
}

export interface Payload = {
    message: string;
};