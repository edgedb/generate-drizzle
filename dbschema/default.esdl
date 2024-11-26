module default {
    global username_prefix: str;

    type Person {
        required first_name: str;
        last_name: str;

        full_name := __source__.first_name ++ ((' ' ++ .last_name) ?? '');
        favorite_genre := (select Genre filter .name = 'Drama' limit 1);
        username := (global username_prefix ?? 'u_') ++ str_lower(.first_name);
    }

    type Genre {
        required name: str;
    }

    global filter_title: str;

    type Content {
        required title: str;
        genre: Genre;

        access policy filter_title
            allow select
            using (global filter_title ?= .title);
        access policy dml allow insert, update, delete;
    }

    type Movie extending Content {
        release_year: int64;
        multi actors: Person {
            role: str;
        };
        director: Person {
            bar: str;
        };

        multi actor_names := __source__.actors.first_name;
        multi similar_to := (select Content);
    }

    type BookContent extending Content {
        required pages: int16;
        multi chapters: str;
    }

    type novel extending Book {
        foo: str;
    }

    module nested {
        type Hello {
            property hello -> str;
        };

        module deep {
            type Rolling {
                property rolling -> str;
            };
        };
    }
}
