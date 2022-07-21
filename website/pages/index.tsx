import type { NextPage } from 'next';
import Head from 'next/head';
import Link from 'next/link';
import { usePostInfo } from '~/providers/postInfo';

const Home: NextPage = () => {
  const [{ posts }] = usePostInfo();

  return (
    <div>
      <Head>
        <title>recardinal&apos; blog</title>
        <meta name="description" content="Generated by create next app" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <ul>
        {posts.map(({ title, route, id }) => (
          <li key={id} className="text-lg">
            <Link href={`/post/${route}`}>{title}</Link>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default Home;

export const myCustomProps = {
  nmsl: 'nmsl',
};
