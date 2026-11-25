//! Interface strings in English, Russian and Ukrainian, and the few phrases built from several parts.

use crate::i18n::{self, Lang, Table};
use crate::model::period::{Period, Unit};

pub struct S {
    // main page
    pub add_sub: &'static str,
    pub settings: &'static str,
    pub back: &'static str,
    pub search_hint: &'static str,
    pub sort_next: &'static str,
    pub sort_price: &'static str,
    pub sort_name: &'static str,
    pub monthly_total: &'static str,
    pub yearly_total: &'static str,
    pub per_year_hint: &'static str,
    pub status_card: &'static str,
    pub active: &'static str,
    pub paused: &'static str,
    pub paused_section: &'static str,
    pub upcoming_title: &'static str,
    pub upcoming_hint: &'static str,
    pub nothing_upcoming: &'static str,
    pub more_upcoming: &'static str,
    pub by_category: &'static str,
    pub no_category: &'static str,
    pub no_matches: &'static str,
    pub today: &'static str,
    pub tomorrow: &'static str,
    pub in_days: [&'static str; 3],
    pub paused_until: &'static str,
    pub empty_title: &'static str,
    pub empty_text: &'static str,
    pub empty_button: &'static str,
    pub open_link: &'static str,
    // rates
    pub rates_from: &'static str,
    pub rates_offline: &'static str,
    pub rates_none: &'static str,
    pub rates_none_hint: &'static str,
    pub rates_refreshing: &'static str,
    pub rates_refresh: &'static str,
    pub rates_missing: &'static str,
    pub total_unavailable: &'static str,
    pub per_currency: &'static str,
    pub rates_needed: &'static str,
    // periods
    pub weekly: &'static str,
    pub monthly: &'static str,
    pub quarterly: &'static str,
    pub yearly: &'static str,
    pub custom: &'static str,
    pub every_one: [&'static str; 3],
    pub every_n: &'static str,
    pub unit_days: [&'static str; 3],
    pub unit_weeks: [&'static str; 3],
    pub unit_months: [&'static str; 3],
    pub unit_day: &'static str,
    pub unit_week: &'static str,
    pub unit_month: &'static str,
    pub short_week: &'static str,
    pub short_month: &'static str,
    pub short_quarter: &'static str,
    pub short_year: &'static str,
    pub short_unit_day: &'static str,
    pub short_unit_week: &'static str,
    pub short_unit_month: &'static str,
    // form
    pub new_sub: &'static str,
    pub edit_sub: &'static str,
    pub f_name: &'static str,
    pub f_name_hint: &'static str,
    pub f_link: &'static str,
    pub f_link_hint: &'static str,
    pub f_price: &'static str,
    pub f_currency: &'static str,
    pub f_search_currency: &'static str,
    pub f_billing: &'static str,
    pub f_every: &'static str,
    pub f_date: &'static str,
    pub f_date_hint: &'static str,
    pub f_category: &'static str,
    pub f_new_category: &'static str,
    pub f_category_hint: &'static str,
    pub f_note: &'static str,
    pub err_name: &'static str,
    pub err_link: &'static str,
    pub err_price: &'static str,
    pub err_date: &'static str,
    pub err_every: &'static str,
    pub save: &'static str,
    pub add: &'static str,
    pub cancel: &'static str,
    pub delete: &'static str,
    pub pause: &'static str,
    pub resume: &'static str,
    pub categories: [&'static str; 6],
    // dialogs
    pub delete_title: &'static str,
    pub delete_body: &'static str,
    pub pause_title: &'static str,
    pub pause_body: &'static str,
    pub pause_until: &'static str,
    pub resume_title: &'static str,
    pub resume_body: &'static str,
    pub corrupt_title: &'static str,
    pub corrupt_body: &'static str,
    pub ok: &'static str,
    pub discard_title: &'static str,
    pub discard_body: &'static str,
    pub discard: &'static str,
    pub keep_editing: &'static str,
    // settings
    pub group_interface: &'static str,
    pub group_money: &'static str,
    pub group_data: &'static str,
    pub first_day: &'static str,
    pub monday: &'static str,
    pub sunday: &'static str,
    pub saturday: &'static str,
    pub default_currency: &'static str,
    pub default_currency_hint: &'static str,
    pub rates_source: &'static str,
    pub rates_updated: &'static str,
    pub rates_refresh_now: &'static str,
    pub open_data_folder: &'static str,
    pub data_hint: &'static str,
    pub about: &'static str,
    pub version: &'static str,
    pub rates_credit: &'static str,
}

pub static TABLE: Table<S> = Table { en: &EN, ru: &RU, uk: &UK };

const EN: S = S {
    add_sub: "Add subscription",
    settings: "Settings",
    back: "Back",
    search_hint: "Search subscriptions",
    sort_next: "Next payment",
    sort_price: "Price",
    sort_name: "Name",
    monthly_total: "Per month",
    yearly_total: "Per year",
    per_year_hint: "Everything you pay in 12 months",
    status_card: "Subscriptions",
    active: "Active",
    paused: "Paused",
    paused_section: "Paused",
    upcoming_title: "Upcoming payments",
    upcoming_hint: "Next 30 days",
    nothing_upcoming: "Nothing is due in the next 30 days.",
    more_upcoming: "and {n} more",
    by_category: "By category",
    no_category: "No category",
    no_matches: "No subscriptions match your search.",
    today: "today",
    tomorrow: "tomorrow",
    in_days: ["in {n} day", "in {n} days", "in {n} days"],
    paused_until: "until {date}",
    empty_title: "No subscriptions yet",
    empty_text: "Add the services you pay for. Cashcycle shows what they cost per month and per year, and what is due next. Everything stays on this computer.",
    empty_button: "Add your first subscription",
    open_link: "Open the service website",
    rates_from: "Rates updated {when}",
    rates_offline: "No connection · using rates from {when}",
    rates_none: "No exchange rates yet",
    rates_none_hint: "Connect to the internet and press refresh. Until then the totals are shown per currency.",
    rates_refreshing: "Updating rates...",
    rates_refresh: "Refresh rates",
    rates_missing: "No rate for {codes}: shown unconverted, left out of the total.",
    total_unavailable: "Total unavailable",
    per_currency: "Per currency, per month",
    rates_needed: "Needs exchange rates",
    weekly: "Weekly",
    monthly: "Monthly",
    quarterly: "Quarterly",
    yearly: "Yearly",
    custom: "Custom",
    every_one: ["every {unit}", "every {unit}", "every {unit}"],
    every_n: "every {n} {unit}",
    unit_days: ["day", "days", "days"],
    unit_weeks: ["week", "weeks", "weeks"],
    unit_months: ["month", "months", "months"],
    unit_day: "days",
    unit_week: "weeks",
    unit_month: "months",
    short_week: "/ wk",
    short_month: "/ mo",
    short_quarter: "/ qtr",
    short_year: "/ yr",
    short_unit_day: "d",
    short_unit_week: "wk",
    short_unit_month: "mo",
    new_sub: "New subscription",
    edit_sub: "Edit subscription",
    f_name: "Name",
    f_name_hint: "Netflix, iCloud, gym...",
    f_link: "Link (optional)",
    f_link_hint: "netflix.com",
    f_price: "Price",
    f_currency: "Currency",
    f_search_currency: "Search code, name or symbol",
    f_billing: "Billing period",
    f_every: "Every",
    f_date: "Next payment, or start date",
    f_date_hint: "A past date works too: the next payment is worked out from it.",
    f_category: "Category",
    f_new_category: "New category...",
    f_category_hint: "Category name",
    f_note: "Note (optional)",
    err_name: "Enter a name.",
    err_link: "This does not look like a web address.",
    err_price: "Enter a price like 9.99 or 9,99.",
    err_date: "Enter a date, for example 2026-12-31.",
    err_every: "Enter a number from 1 to 999.",
    save: "Save",
    add: "Add",
    cancel: "Cancel",
    delete: "Delete",
    pause: "Pause",
    resume: "Resume",
    categories: ["Video", "Music", "Software", "Cloud", "Gaming", "Other"],
    delete_title: "Delete {name}?",
    delete_body: "This removes the subscription from Cashcycle for good. If you only want to stop counting it, pause it instead: a paused subscription stays in the list and is left out of every total.",
    pause_title: "Pause {name}?",
    pause_body: "A paused subscription is left out of the totals and the upcoming payments. You can resume it at any time.",
    pause_until: "Resume by itself on a date",
    resume_title: "Resume {name}",
    resume_body: "When is the next payment?",
    corrupt_title: "Your data file could not be read",
    corrupt_body: "Cashcycle kept the unreadable file as {file} in the data folder and started with an empty list. Your other settings are untouched.",
    ok: "OK",
    discard_title: "Discard changes?",
    discard_body: "You have changed this subscription and not saved it yet.",
    discard: "Discard",
    keep_editing: "Keep editing",
    group_interface: "Interface",
    group_money: "Money",
    group_data: "Data",
    first_day: "First day of the week",
    monday: "Monday",
    sunday: "Sunday",
    saturday: "Saturday",
    default_currency: "Currency for new subscriptions",
    default_currency_hint: "The total card has its own currency, chosen right on the card.",
    rates_source: "Exchange rates",
    rates_updated: "Updated",
    rates_refresh_now: "Update now",
    open_data_folder: "Open data folder",
    data_hint: "Subscriptions and rates are stored in this folder. Nothing leaves your computer except the exchange-rate request.",
    about: "About",
    version: "Version",
    rates_credit: "Exchange rates by Exchange Rate API",
};

const RU: S = S {
    add_sub: "Добавить подписку",
    settings: "Настройки",
    back: "Назад",
    search_hint: "Поиск по подпискам",
    sort_next: "Ближайший платёж",
    sort_price: "Цена",
    sort_name: "Название",
    monthly_total: "В месяц",
    yearly_total: "В год",
    per_year_hint: "Всё, что вы платите за 12 месяцев",
    status_card: "Подписки",
    active: "Активные",
    paused: "На паузе",
    paused_section: "На паузе",
    upcoming_title: "Ближайшие платежи",
    upcoming_hint: "Следующие 30 дней",
    nothing_upcoming: "В ближайшие 30 дней платежей нет.",
    more_upcoming: "и ещё {n}",
    by_category: "По категориям",
    no_category: "Без категории",
    no_matches: "Ничего не найдено.",
    today: "сегодня",
    tomorrow: "завтра",
    in_days: ["через {n} день", "через {n} дня", "через {n} дней"],
    paused_until: "до {date}",
    empty_title: "Подписок пока нет",
    empty_text: "Добавьте сервисы, за которые вы платите. Cashcycle покажет, сколько они стоят в месяц и в год и что оплачивать дальше. Всё хранится на этом компьютере.",
    empty_button: "Добавить первую подписку",
    open_link: "Открыть сайт сервиса",
    rates_from: "Курсы обновлены {when}",
    rates_offline: "Нет соединения · курсы от {when}",
    rates_none: "Курсов валют пока нет",
    rates_none_hint: "Подключитесь к интернету и нажмите «Обновить». А пока суммы показаны по валютам.",
    rates_refreshing: "Обновляю курсы...",
    rates_refresh: "Обновить курсы",
    rates_missing: "Нет курса для {codes}: показано без пересчёта, в итог не входит.",
    total_unavailable: "Итог недоступен",
    per_currency: "По валютам, в месяц",
    rates_needed: "Нужны курсы валют",
    weekly: "Каждую неделю",
    monthly: "Каждый месяц",
    quarterly: "Каждый квартал",
    yearly: "Каждый год",
    custom: "Свой период",
    every_one: ["каждый день", "каждую неделю", "каждый месяц"],
    every_n: "каждые {n} {unit}",
    unit_days: ["день", "дня", "дней"],
    unit_weeks: ["неделю", "недели", "недель"],
    unit_months: ["месяц", "месяца", "месяцев"],
    unit_day: "дней",
    unit_week: "недель",
    unit_month: "месяцев",
    short_week: "/ нед.",
    short_month: "/ мес.",
    short_quarter: "/ кв.",
    short_year: "/ год",
    short_unit_day: "дн.",
    short_unit_week: "нед.",
    short_unit_month: "мес.",
    new_sub: "Новая подписка",
    edit_sub: "Изменить подписку",
    f_name: "Название",
    f_name_hint: "Netflix, iCloud, спортзал...",
    f_link: "Ссылка (необязательно)",
    f_link_hint: "netflix.com",
    f_price: "Цена",
    f_currency: "Валюта",
    f_search_currency: "Код, название или символ",
    f_billing: "Период оплаты",
    f_every: "Каждые",
    f_date: "Следующий платёж или дата начала",
    f_date_hint: "Можно указать и прошедшую дату: ближайший платёж посчитается от неё.",
    f_category: "Категория",
    f_new_category: "Новая категория...",
    f_category_hint: "Название категории",
    f_note: "Заметка (необязательно)",
    err_name: "Введите название.",
    err_link: "Это не похоже на веб-адрес.",
    err_price: "Введите цену, например 9.99 или 9,99.",
    err_date: "Введите дату, например 31.12.2026.",
    err_every: "Введите число от 1 до 999.",
    save: "Сохранить",
    add: "Добавить",
    cancel: "Отмена",
    delete: "Удалить",
    pause: "Пауза",
    resume: "Возобновить",
    categories: ["Видео", "Музыка", "Программы", "Облако", "Игры", "Другое"],
    delete_title: "Удалить «{name}»?",
    delete_body: "Подписка будет удалена из Cashcycle навсегда. Если нужно просто не учитывать её, поставьте на паузу: она останется в списке, но не попадёт ни в один итог.",
    pause_title: "Поставить «{name}» на паузу?",
    pause_body: "Подписка на паузе не входит в итоги и в ближайшие платежи. Её можно возобновить в любой момент.",
    pause_until: "Возобновить самой в выбранный день",
    resume_title: "Возобновить «{name}»",
    resume_body: "Когда следующий платёж?",
    corrupt_title: "Не удалось прочитать файл с данными",
    corrupt_body: "Cashcycle сохранил нечитаемый файл как {file} в папке данных и начал с пустого списка. Остальные настройки не затронуты.",
    ok: "Понятно",
    discard_title: "Отменить изменения?",
    discard_body: "Вы изменили подписку, но ещё не сохранили её.",
    discard: "Отменить изменения",
    keep_editing: "Продолжить",
    group_interface: "Интерфейс",
    group_money: "Деньги",
    group_data: "Данные",
    first_day: "Первый день недели",
    monday: "Понедельник",
    sunday: "Воскресенье",
    saturday: "Суббота",
    default_currency: "Валюта новых подписок",
    default_currency_hint: "У карточки с итогом своя валюта, её выбирают прямо на карточке.",
    rates_source: "Курсы валют",
    rates_updated: "Обновлено",
    rates_refresh_now: "Обновить",
    open_data_folder: "Открыть папку данных",
    data_hint: "Подписки и курсы хранятся в этой папке. Из компьютера уходит только запрос курсов валют.",
    about: "О программе",
    version: "Версия",
    rates_credit: "Курсы валют: Exchange Rate API",
};

const UK: S = S {
    add_sub: "Додати підписку",
    settings: "Налаштування",
    back: "Назад",
    search_hint: "Пошук за підписками",
    sort_next: "Найближчий платіж",
    sort_price: "Ціна",
    sort_name: "Назва",
    monthly_total: "На місяць",
    yearly_total: "На рік",
    per_year_hint: "Усе, що ви платите за 12 місяців",
    status_card: "Підписки",
    active: "Активні",
    paused: "На паузі",
    paused_section: "На паузі",
    upcoming_title: "Найближчі платежі",
    upcoming_hint: "Наступні 30 днів",
    nothing_upcoming: "Найближчі 30 днів платежів немає.",
    more_upcoming: "і ще {n}",
    by_category: "За категоріями",
    no_category: "Без категорії",
    no_matches: "Нічого не знайдено.",
    today: "сьогодні",
    tomorrow: "завтра",
    in_days: ["через {n} день", "через {n} дні", "через {n} днів"],
    paused_until: "до {date}",
    empty_title: "Підписок поки немає",
    empty_text: "Додайте сервіси, за які ви платите. Cashcycle покаже, скільки вони коштують на місяць і на рік та що оплачувати далі. Усе зберігається на цьому комп'ютері.",
    empty_button: "Додати першу підписку",
    open_link: "Відкрити сайт сервісу",
    rates_from: "Курси оновлено {when}",
    rates_offline: "Немає з'єднання · курси від {when}",
    rates_none: "Курсів валют поки немає",
    rates_none_hint: "Підключіться до інтернету й натисніть «Оновити». А поки суми показано за валютами.",
    rates_refreshing: "Оновлюю курси...",
    rates_refresh: "Оновити курси",
    rates_missing: "Немає курсу для {codes}: показано без перерахунку, у підсумок не входить.",
    total_unavailable: "Підсумок недоступний",
    per_currency: "За валютами, на місяць",
    rates_needed: "Потрібні курси валют",
    weekly: "Щотижня",
    monthly: "Щомісяця",
    quarterly: "Щокварталу",
    yearly: "Щороку",
    custom: "Свій період",
    every_one: ["щодня", "щотижня", "щомісяця"],
    every_n: "кожні {n} {unit}",
    unit_days: ["день", "дні", "днів"],
    unit_weeks: ["тиждень", "тижні", "тижнів"],
    unit_months: ["місяць", "місяці", "місяців"],
    unit_day: "днів",
    unit_week: "тижнів",
    unit_month: "місяців",
    short_week: "/ тиж.",
    short_month: "/ міс.",
    short_quarter: "/ кв.",
    short_year: "/ рік",
    short_unit_day: "дн.",
    short_unit_week: "тиж.",
    short_unit_month: "міс.",
    new_sub: "Нова підписка",
    edit_sub: "Змінити підписку",
    f_name: "Назва",
    f_name_hint: "Netflix, iCloud, спортзал...",
    f_link: "Посилання (необов'язково)",
    f_link_hint: "netflix.com",
    f_price: "Ціна",
    f_currency: "Валюта",
    f_search_currency: "Код, назва або символ",
    f_billing: "Період оплати",
    f_every: "Кожні",
    f_date: "Наступний платіж або дата початку",
    f_date_hint: "Можна вказати й минулу дату: найближчий платіж порахується від неї.",
    f_category: "Категорія",
    f_new_category: "Нова категорія...",
    f_category_hint: "Назва категорії",
    f_note: "Нотатка (необов'язково)",
    err_name: "Введіть назву.",
    err_link: "Це не схоже на веб-адресу.",
    err_price: "Введіть ціну, наприклад 9.99 або 9,99.",
    err_date: "Введіть дату, наприклад 31.12.2026.",
    err_every: "Введіть число від 1 до 999.",
    save: "Зберегти",
    add: "Додати",
    cancel: "Скасувати",
    delete: "Видалити",
    pause: "Пауза",
    resume: "Відновити",
    categories: ["Відео", "Музика", "Програми", "Хмара", "Ігри", "Інше"],
    delete_title: "Видалити «{name}»?",
    delete_body: "Підписку буде видалено з Cashcycle назавжди. Якщо треба просто не враховувати її, поставте на паузу: вона залишиться в списку, але не потрапить в жоден підсумок.",
    pause_title: "Поставити «{name}» на паузу?",
    pause_body: "Підписка на паузі не входить до підсумків і найближчих платежів. Її можна відновити будь-коли.",
    pause_until: "Відновити самій у вибраний день",
    resume_title: "Відновити «{name}»",
    resume_body: "Коли наступний платіж?",
    corrupt_title: "Не вдалося прочитати файл із даними",
    corrupt_body: "Cashcycle зберіг нечитабельний файл як {file} у теці даних і почав з порожнього списку. Решту налаштувань не зачеплено.",
    ok: "Зрозуміло",
    discard_title: "Скасувати зміни?",
    discard_body: "Ви змінили підписку, але ще не зберегли її.",
    discard: "Скасувати зміни",
    keep_editing: "Продовжити",
    group_interface: "Інтерфейс",
    group_money: "Гроші",
    group_data: "Дані",
    first_day: "Перший день тижня",
    monday: "Понеділок",
    sunday: "Неділя",
    saturday: "Субота",
    default_currency: "Валюта нових підписок",
    default_currency_hint: "Картка з підсумком має власну валюту, її вибирають просто на картці.",
    rates_source: "Курси валют",
    rates_updated: "Оновлено",
    rates_refresh_now: "Оновити",
    open_data_folder: "Відкрити теку даних",
    data_hint: "Підписки й курси зберігаються в цій теці. З комп'ютера йде лише запит курсів валют.",
    about: "Про програму",
    version: "Версія",
    rates_credit: "Курси валют: Exchange Rate API",
};

/// "weekly", "every 3 weeks", "каждые 2 месяца": a billing period in words.
pub fn period_text(lang: Lang, p: Period) -> String {
    let t = TABLE.get(lang);
    match p.sanitized() {
        Period::Weekly => t.weekly.to_string(),
        Period::Monthly => t.monthly.to_string(),
        Period::Quarterly => t.quarterly.to_string(),
        Period::Yearly => t.yearly.to_string(),
        Period::Custom { every, unit } => {
            let (forms, idx) = match unit {
                Unit::Day => (&t.unit_days, 0),
                Unit::Week => (&t.unit_weeks, 1),
                Unit::Month => (&t.unit_months, 2),
            };
            if every == 1 {
                return match lang {
                    Lang::En => t.every_one[0].replace("{unit}", forms[0]),
                    _ => t.every_one[idx].to_string(),
                };
            }
            let unit_word = forms[i18n::plural_index(lang, every as usize)];
            t.every_n.replace("{n}", &every.to_string()).replace("{unit}", unit_word)
        }
    }
}

/// The short suffix after a price: "/ mo", "/ мес.", "/ 2 wk".
pub fn period_short(lang: Lang, p: Period) -> String {
    let t = TABLE.get(lang);
    match p.sanitized() {
        Period::Weekly => t.short_week.to_string(),
        Period::Monthly => t.short_month.to_string(),
        Period::Quarterly => t.short_quarter.to_string(),
        Period::Yearly => t.short_year.to_string(),
        Period::Custom { every, unit } => {
            let u = match unit {
                Unit::Day => t.short_unit_day,
                Unit::Week => t.short_unit_week,
                Unit::Month => t.short_unit_month,
            };
            if every == 1 {
                format!("/ {u}")
            } else {
                format!("/ {every} {u}")
            }
        }
    }
}

/// "today", "tomorrow", "in 3 days" for up to two months ahead; empty after that (the date is enough).
pub fn relative_days(lang: Lang, days: i64) -> String {
    let t = TABLE.get(lang);
    match days {
        0 => t.today.to_string(),
        1 => t.tomorrow.to_string(),
        2..=60 => i18n::plural(lang, days as usize, &t.in_days),
        _ => String::new(),
    }
}

/// A built-in category in the interface language; user-defined names are shown as they are.
pub fn category_name(lang: Lang, stored: &str) -> String {
    let t = TABLE.get(lang);
    match crate::model::types::BUILTIN_CATEGORIES.iter().position(|b| *b == stored) {
        Some(i) => t.categories[i].to_string(),
        None if stored.is_empty() => t.no_category.to_string(),
        None => stored.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periods_read_naturally() {
        let c = |every, unit| Period::Custom { every, unit };
        assert_eq!(period_text(Lang::En, c(3, Unit::Week)), "every 3 weeks");
        assert_eq!(period_text(Lang::En, c(1, Unit::Month)), "every month");
        assert_eq!(period_text(Lang::En, c(1, Unit::Day)), "every day");
        assert_eq!(period_text(Lang::Ru, c(2, Unit::Month)), "каждые 2 месяца");
        assert_eq!(period_text(Lang::Ru, c(5, Unit::Week)), "каждые 5 недель");
        assert_eq!(period_text(Lang::Ru, c(21, Unit::Day)), "каждые 21 день");
        assert_eq!(period_text(Lang::Ru, c(1, Unit::Week)), "каждую неделю");
        assert_eq!(period_text(Lang::Uk, c(3, Unit::Day)), "кожні 3 дні");
        assert_eq!(period_text(Lang::Uk, c(1, Unit::Month)), "щомісяця");
        assert_eq!(period_short(Lang::En, Period::Monthly), "/ mo");
        assert_eq!(period_short(Lang::Ru, c(2, Unit::Week)), "/ 2 нед.");
        assert_eq!(period_short(Lang::Uk, c(1, Unit::Day)), "/ дн.");
    }

    #[test]
    fn relative_dates() {
        assert_eq!(relative_days(Lang::En, 0), "today");
        assert_eq!(relative_days(Lang::En, 1), "tomorrow");
        assert_eq!(relative_days(Lang::En, 3), "in 3 days");
        assert_eq!(relative_days(Lang::Ru, 2), "через 2 дня");
        assert_eq!(relative_days(Lang::Ru, 5), "через 5 дней");
        assert_eq!(relative_days(Lang::Ru, 21), "через 21 день");
        assert_eq!(relative_days(Lang::Uk, 22), "через 22 дні");
        assert_eq!(relative_days(Lang::En, 90), "");
    }

    #[test]
    fn category_names_are_translated_only_for_builtins() {
        assert_eq!(category_name(Lang::Ru, "Video"), "Видео");
        assert_eq!(category_name(Lang::Uk, "Gaming"), "Ігри");
        assert_eq!(category_name(Lang::En, "Books"), "Books");
        assert_eq!(category_name(Lang::Ru, ""), "Без категории");
    }

    /// Every string is filled in every language, placeholders match the English text.
    #[test]
    fn every_string_is_filled_and_placeholders_match() {
        fn flat(s: &S) -> Vec<(&'static str, String)> {
            let mut v: Vec<(&'static str, String)> = Vec::new();
            macro_rules! f { ($($n:ident),* $(,)?) => { $( v.push((stringify!($n), s.$n.to_string())); )* }; }
            f!(add_sub, settings, back, search_hint, sort_next, sort_price, sort_name, monthly_total, yearly_total, per_year_hint,
               status_card, active, paused, paused_section, upcoming_title, upcoming_hint, nothing_upcoming, more_upcoming,
               by_category, no_category, no_matches, today, tomorrow, paused_until, empty_title, empty_text, empty_button,
               open_link, rates_from, rates_offline, rates_none, rates_none_hint, rates_refreshing, rates_refresh, rates_missing,
               total_unavailable, per_currency, rates_needed, weekly, monthly, quarterly, yearly, custom, every_n, unit_day, unit_week,
               unit_month, short_week, short_month, short_quarter, short_year, short_unit_day, short_unit_week, short_unit_month, new_sub,
               edit_sub, f_name, f_name_hint, f_link, f_link_hint, f_price, f_currency, f_search_currency, f_billing, f_every, f_date,
               f_date_hint, f_category, f_new_category, f_category_hint, f_note, err_name, err_link, err_price, err_date, err_every, save,
               add, cancel, delete, pause, resume, delete_title, delete_body, pause_title, pause_body, pause_until, resume_title,
               resume_body, corrupt_title, corrupt_body, ok, discard_title, discard_body, discard, keep_editing, group_interface, group_money, group_data, first_day, monday, sunday,
               saturday, default_currency, default_currency_hint, rates_source, rates_updated,
               rates_refresh_now, open_data_folder, data_hint, about, version, rates_credit);
            for (i, x) in s.in_days.iter().enumerate() { v.push(("in_days", format!("{i}:{x}"))); }
            for (i, x) in s.every_one.iter().enumerate() { v.push(("every_one", format!("{i}:{x}"))); }
            for (i, x) in s.unit_days.iter().enumerate() { v.push(("unit_days", format!("{i}:{x}"))); }
            for (i, x) in s.unit_weeks.iter().enumerate() { v.push(("unit_weeks", format!("{i}:{x}"))); }
            for (i, x) in s.unit_months.iter().enumerate() { v.push(("unit_months", format!("{i}:{x}"))); }
            for (i, x) in s.categories.iter().enumerate() { v.push(("categories", format!("{i}:{x}"))); }
            v
        }
        fn holes(text: &str) -> Vec<String> {
            let mut out = Vec::new();
            let mut rest = text;
            while let Some(a) = rest.find('{') {
                let Some(b) = rest[a..].find('}') else { break };
                out.push(rest[a..a + b + 1].to_string());
                rest = &rest[a + b + 1..];
            }
            out.sort();
            out
        }
        let en = flat(&EN);
        for (name, lang) in [("ru", &RU), ("uk", &UK)] {
            let other = flat(lang);
            assert_eq!(en.len(), other.len());
            for ((k, e), (k2, o)) in en.iter().zip(&other) {
                assert_eq!(k, k2);
                let body = o.split_once(':').filter(|(i, _)| i.len() == 1).map(|x| x.1).unwrap_or(o);
                assert!(!body.trim().is_empty(), "{name}.{k} is empty");
                let ebody = e.split_once(':').filter(|(i, _)| i.len() == 1).map(|x| x.1).unwrap_or(e);
                if *k != "every_one" {
                    assert_eq!(holes(ebody), holes(body), "{name}.{k}: placeholders differ");
                }
                if name != "en" && *k != "f_name_hint" && *k != "f_link_hint" && *k != "ok" && !ebody.chars().all(|c| !c.is_alphabetic()) {
                    // A translation that is byte-identical to the English text is almost certainly forgotten.
                    assert!(ebody != body || ebody.len() <= 3, "{name}.{k} looks untranslated: {body}");
                }
            }
        }
        for s in [&EN, &RU, &UK] {
            assert!(s.in_days.iter().all(|x| x.contains("{n}")));
            assert!(s.every_n.contains("{n}") && s.every_n.contains("{unit}"));
        }
    }
}
