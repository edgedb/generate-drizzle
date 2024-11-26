import { sql } from 'drizzle-orm';
import { pgSchema, text, uuid } from 'drizzle-orm/pg-core';


export const rollingTable = pgSchema("public::nested::deep").table("Rolling", {
  id: uuid().notNull().default(sql.raw("std::uuid_generate_v1mc()")),
  rolling: text(),
});
