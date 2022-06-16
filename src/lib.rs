pub mod message_box {
    use rfd::{MessageButtons, MessageDialog, MessageLevel};

    pub enum ErrorMsgBox {
        EmptyInputField,
        EmptyOutputField,
        EmptyListField,
    }

    impl ErrorMsgBox {
        pub fn value(&self) -> &str {
            match self {
                ErrorMsgBox::EmptyInputField => "The input field can not be empty.",
                ErrorMsgBox::EmptyOutputField => "The output field can not be empty.",
                ErrorMsgBox::EmptyListField => "The list field can not be empty.",
            }
        }
    }

    pub enum StateMsgBox {
        Success,
    }

    impl StateMsgBox {
        pub fn value(&self) -> &str {
            match self {
                StateMsgBox::Success => "Success!",
            }
        }

        pub fn show(&self) {
            MessageDialog::new()
                .set_title("Success")
                .set_description(&self.value())
                .set_level(MessageLevel::Info)
                .set_buttons(MessageButtons::Ok)
                .show();
        }
    }

    /// The default message for when a field is required but empty.
    pub fn empty_field(error: ErrorMsgBox) {
        MessageDialog::new()
            .set_title("Error")
            .set_description(error.value())
            .set_level(MessageLevel::Error)
            .set_buttons(MessageButtons::Ok)
            .show();
    }

    pub fn state_msg(state: StateMsgBox) {
        MessageDialog::new()
            .set_title("Success")
            .set_description(state.value())
            .set_level(MessageLevel::Info)
            .set_buttons(MessageButtons::Ok)
            .show();
    }
}
