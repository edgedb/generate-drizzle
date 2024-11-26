import { sql } from 'drizzle-orm';
import { pgSchema, text, uuid } from 'drizzle-orm/pg-core';


export const helloTable = pgSchema("public::nested").table("Hello", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
  hello: text(),
});
