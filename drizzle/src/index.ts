import 'dotenv/config';
import { drizzle } from 'drizzle-orm/node-postgres';
import { eq } from 'drizzle-orm';
import { genreTable, movieTable } from './db/schema/default';
import * as schema from './db/schema/default';

const db = drizzle({ schema, connection: process.env.DATABASE_URL! });

async function main() {
    const genre: typeof genreTable.$inferInsert = {
        name: 'Crime',
    };
    const [genre_inserted] = await db.insert(genreTable).values(genre).returning();

    const movie: typeof movieTable.$inferInsert = {
        title: 'Here and There',
        release_year: 2005,
        genre_id: genre_inserted.id
    };

    await db.insert(movieTable).values(movie);
    console.log('New movie created!')

    const movies = await db.query.movieTable.findMany({ with: { genre: true } });
    console.log('movies =', movies)
    /*
    const users: {
      id: number;
      name: string;
      age: number;
      email: string;
    }[]
    */

    await db
        .update(movieTable)
        .set({
            release_year: 2007,
        })
        .where(eq(movieTable.title, movie.title));
    console.log('Movie info updated!')

    const movies2 = await db.query.movieTable.findMany({ with: { genre: true } });
    console.log('movies =', movies2)

    await db.delete(movieTable).where(eq(movieTable.title, movie.title));
    console.log('Movie deleted!')

    const genres = await db.query.genreTable.findMany();
    console.log('genres =', genres)
}

main();